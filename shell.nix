with import <nixpkgs> {};

mkShell{
  name="nju-schedule-ics test environment";

  buildInputs=[
    redis
  ];

  // TODO: figure out what exactly does the arguments mean
  shellHook = ''
    redis-server --dir redis --dbfilename cokies.rdb --save 10 1
  ''
}
