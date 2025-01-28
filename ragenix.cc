#include <primops.hh>

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
        .atPos(pos)
        .debugThrow();
  }

  return value;
}

std::string mkString(RageString rageString) {
  std::string out(rageString.string, rageString.len);
  dealloc(rageString);
  return out;
}

void decryptPrimOp(nix::EvalState &state, const nix::PosIdx pos,
                   nix::Value **args, nix::Value &out) {
  auto key = getArg(state, pos, args[0], nix::nPath);
  auto path = getArg(state, pos, args[1], nix::nPath);
  auto status = uint8_t{0};
  auto decrypted =
      mkString(decrypt(key.payload.path.path, path.payload.path.path, status));

  if (status == 0) {
    try {
      auto expr = state.parseExprFromString(decrypted, path.path());
      state.eval(expr, out);
    } catch (nix::UndefinedVarError &) {
      out.mkString(decrypted);
    }
  } else {
    state.error<nix::EvalError>("could not decrypt %1%", path.payload.path.path)
        .withTrace(pos, decrypted)
        .atPos(pos)
        .debugThrow();
  }
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
