{
  pkgs,
}:

pkgs.python3Packages.buildPythonApplication {
  pname = "dap";
  version = "0.1.0";

  src = ./.;
  pyproject = true;

  build-system = [
    pkgs.python3Packages.setuptools
  ];

  nativeCheckInputs = [
    pkgs.python3Packages.pytest
  ];

  checkPhase = ''
    runHook preCheck

    export PYTHONPATH=$PWD:$PYTHONPATH

    pytest dap_tests.py

    runHook postCheck
  '';

  meta = {
    description = "Diff Apply Tool";
    mainProgram = "dap";
  };
}
