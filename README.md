### Z
A strict superset of C aimed at making large scale programming a little easier while keeping the low-level control C allows

## Features
* Classes
* Compiles into C
* Full C preprocessor support
* Any C code is valid Z code
## Bugs
- Currently flattens nested classes
- Functions in classes that do not modify themself come with a slight overhead since self is still passed as a param

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
