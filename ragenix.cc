#include <config-global.hh>
#include <config.h>
#include <eval-settings.hh>
#include <globals.hh>
#include <primops.hh>

struct Settings : nix::Config {
  nix::Setting<nix::Path> pubKey{this, nix::settings.nixConfDir + "/key.pub",
                                 "pubKey",
                                 "The public key to use for decryption"};
};

Settings settings;

void decryptPrimOp(nix::EvalState &state, const nix::PosIdx pos,
                   nix::Value **args, nix::Value &out) {
  auto pathArg = *args[0];
  state.forceValue(pathArg, pos);
  if (pathArg.type() != nix::nPath) {
    state
        .error<nix::TypeError>("value is %1% while a path was expectd",
                               nix::showType(pathArg))
        .atPos(pos)
        .debugThrow();
  }

  auto pubKeyArg = *args[1];
  state.forceValue(pubKeyArg, pos);
  if (pubKeyArg.type() != nix::nString) {
    state
        .error<nix::TypeError>("value is %1% while a string was expectd",
                               nix::showType(pubKeyArg))
        .atPos(pos)
        .debugThrow();
  }

  auto path = settings.pubKey.get();
  auto exprStr = std::string{path};
  exprStr += pubKeyArg.string_view();
  auto expr = state.parseExprFromString(exprStr, pathArg.path());
  state.eval(expr, out);
}

extern "C" void nix_plugin_entry() {
  nix::RegisterPrimOp primop({
      .name = "__decrypt",
      .args = {"path", "pubkey"},
      .arity = 2,
      .doc = "Decrypt an evaluate a file",
      .fun = decryptPrimOp,
      .experimentalFeature = {},
  });

  nix::GlobalConfig::Register config(&settings);
}
