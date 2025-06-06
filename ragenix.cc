#include <nix/expr/primops.hh>

extern "C" {
struct RageString {
  const char *string;
  const std::size_t len;
  const std::size_t cap;
};
RageString decrypt(const char *key, const char *path, uint8_t &status);
void dealloc(const RageString &string);
}

nix::Value getArg(nix::EvalState &state, const nix::PosIdx pos, nix::Value *arg,
                  const nix::ValueType type) {
  if (arg == NULL) {
    state
        .error<nix::MissingArgumentError>("expected %1% parameter",
                                          nix::showType(type))
        .atPos(pos)
        .debugThrow();
  }

  auto value = *arg;
  state.forceValue(value, pos);
  if (value.type() != type) {
    state
        .error<nix::TypeError>("value is %1% while %2% was expected",
                               nix::showType(value), nix::showType(type))
        .atPos(arg->determinePos(pos))
        .debugThrow();
  }

  return value;
}

void decryptPrimOp(nix::EvalState &state, const nix::PosIdx pos,
                   nix::Value **args, nix::Value &out) {
  auto output = state.buildBindings(1);
  auto key = getArg(state, pos, args[0], nix::nPath);
  auto path = getArg(state, pos, args[1], nix::nPath);
  auto status = uint8_t{0};
  auto decrypted =
      decrypt(key.payload.path.path, path.payload.path.path, status);
  auto payload = std::string(decrypted.string, decrypted.len);

  if (status == 0) {
    try {
      auto expr = state.parseExprFromString(payload, path.path());
      auto ok = state.allocValue();
      state.eval(expr, *ok);
      output.insert(state.symbols.create("ok"), ok);
    } catch (nix::UndefinedVarError &) {
      output.alloc("ok").mkString(payload);
    } catch (const std::exception &e) {
      output.alloc("err").mkString(e.what());
    }
  } else {
    auto result = std::string();
    auto strFront = std::string_view{"could not decrypt "};
    auto strPath = std::string_view{path.payload.path.path};
    auto strColon = std::string_view{": "};

    result.reserve(strFront.length() + strPath.length() + strPath.length() +
                   strColon.length() + payload.length());

    result.append(strFront);
    result.append(strPath);
    result.append(strColon);
    result.append(payload);

    output.alloc("err").mkString(result);
  }
  out.mkAttrs(output.finish());
}

extern "C" void cpp_entry() {
  nix::RegisterPrimOp primop({
      .name = "__decrypt",
      .args = {"key", "path"},
      .arity = 2,
      .doc = "Decrypt an evaluate a file",
      .fun = decryptPrimOp,
      .experimentalFeature = {},
  });
}
