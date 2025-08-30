[<img width="128" height="128" alt="z-icon" src="https://github.com/user-attachments/assets/2fe29c66-69cc-473f-a9eb-cf4eb83322e4" />](https://github.com/hydrophobis/Z-Compiler/blob/main/z-icon.png)<br>
A strict superset of C aimed at making large scale programming a little easier while keeping the low-level control C allows

## Features
* Classes
* Compiles into C
* Full C preprocessor support
* Any C code is valid Z code
## Bugs
- Currently flattens nested classes
- Functions in classes that do not modify themself come with a slight overhead since self is still passed as a param

## Requirements
* gcc

## Usage
Define classes with the class keyword
```CPP
class demo {

}
```
Classes can contain variables like structs
```CPP
class demo {
  int f;
}
```
You can define functions within classes and call them with a '.'
```CPP
class demo {
  int demofunc(char c){
    return c * 2;
  }
}
int main(){
  demo demo_inst;
  demo_inst.demofunc('3');
}
```
Classes can modify themselves within a function by calling self
```CPP
class demo {
  int i;
  int demofunc(int inc){
    self.i = i + inc;
    return self.i;
  }
}
int main(){
  demo demo_inst;
  demo_inst.demofunc('3');
}
```
Include C files using #include or include Z files using #import because Z files must be transpiled before included while C files cannot be
```CPP
#include <stdio.h>
#import <localfile.z>
```
Use any C preprocessor directive
```CPP
#define DEMO
#ifdef DEMO
#undef DEMO
#else
#endif
...
```
Operator overloading ("+", "-", "*", "/", "==", "!=", "<", ">", "<=", ">=", "+=", "-=", "*=", "/=")
```CPP
class demo {
  int x;
  demo operator+(demo other){
    return self.x + other.x;
  }
}
```