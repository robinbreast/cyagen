
build test
```
$ cmake -S . -B build
$ cmake --build build
```

run test
```
$ cd build && ctest --rerun-failed --output-on-failure
```

Refer to the [link](https://google.github.io/googletest/quickstart-cmake.html)

