#include <stdio.h>

static const int my_config = 10;

int add(int a, int b) {
    return a + b;
}

int mul(int a, int b) {
    return a * b;
}

int get_config(void)
{
    return my_config;
}

