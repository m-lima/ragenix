#include <primops.hh>

extern "C" {
struct RageString {
  const char *string;
  const std::size_t len;
  const std::size_t cap;
};
RageString decrypt(const char *path, const char *pubKey, uint8_t &status);
void dealloc(const RageString &string);
}

struct RageStringWrapper {
public:
  RageStringWrapper(const RageString string) : mString{string} {}
  ~RageStringWrapper() {
    if (mDestruct) {
      dealloc(mString);
    }
  }
  inline void leak() { mDestruct = false; }
  inline const char *str() { return mString.string; }

private:
  const RageString mString;
  bool mDestruct = true;
};

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

void decryptPrimOp(nix::EvalState &state, const nix::PosIdx pos,
                   nix::Value **args, nix::Value &out) {
  auto path = getArg(state, pos, args[0], nix::nPath);
  auto pubKey = getArg(state, pos, args[1], nix::nPath);
  auto status = uint8_t{0};
  auto output = RageStringWrapper{
      decrypt(path.payload.path.path, pubKey.payload.path.path, status)};

  if (status == 0) {
    try {
      auto expr = state.parseExprFromString(output.str(), path.path());
      state.eval(expr, out);
    } catch (nix::UndefinedVarError &) {
      out.mkString(output.str());
      output.leak();
    }
  } else {
    state.error<nix::EvalError>("could not decrypt %1%", path.payload.path.path)
        .withTrace(pos, output.str())
        .atPos(pos)
        .debugThrow();
  }
}

extern "C" void cpp_entry() {
  nix::RegisterPrimOp primop({
      .name = "__decrypt",
      .args = {"path", "pubkey"},
      .arity = 2,
      .doc = "Decrypt an evaluate a file",
      .fun = decryptPrimOp,
      .experimentalFeature = {},
  });
}
