<!--
Licensed to the Apache Software Foundation (ASF) under one
or more contributor license agreements.  See the NOTICE file
distributed with this work for additional information
regarding copyright ownership.  The ASF licenses this file
to you under the Apache License, Version 2.0 (the
"License"); you may not use this file except in compliance
with the License.  You may obtain a copy of the License at

  http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing,
software distributed under the License is distributed on an
"AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
KIND, either express or implied.  See the License for the
specific language governing permissions and limitations
under the License.
-->

# UniLang Standard Library — API Reference

Every function listed here is pre-registered in the runtime VM and available in any `.uniL` program without imports.

---

## Table of Contents

1. [I/O](#1-io)
2. [Type Conversion](#2-type-conversion)
3. [Type Checking](#3-type-checking)
4. [Utility](#4-utility)
5. [Aggregates](#5-aggregates)
6. [Character Conversion](#6-character-conversion)
7. [Collection Constructors](#7-collection-constructors)
8. [Collections](#8-collections)
9. [String](#9-string)
10. [Math](#10-math)
11. [Math Constants](#10a-math-constants)
12. [JSON](#11-json)
13. [Time](#12-time)
14. [DateTime](#13-datetime)
15. [File](#14-file)
16. [Environment](#15-environment)
17. [HTTP Client](#16-http-client)
18. [HTTP Server](#17-http-server)
19. [Random](#18-random)
20. [Regex](#19-regex)
21. [UUID](#20-uuid)
22. [Base64](#21-base64)
23. [Crypto](#22-crypto)
24. [CSV](#23-csv)
25. [Global Constants](#24-global-constants)

---

## 1. I/O

### `print(...values) → None`

Prints all arguments to stdout separated by spaces, followed by a newline. Also registered as `println`.

| Argument | Type | Description |
|----------|------|-------------|
| `...values` | any (variadic) | One or more values to print |

**Returns:** `None`

```unilang
print("Hello", "World")     # Hello World
println("Count:", 42)       # Count: 42
```

---

### `input(prompt?) → str`

Reads a line from stdin. If `prompt` is provided it is printed first (without a newline) to prompt the user.

| Argument | Type | Description |
|----------|------|-------------|
| `prompt` | str (optional) | Text displayed before reading input |

**Returns:** `str` — the line entered by the user, with trailing newline stripped.

```unilang
name = input("Enter your name: ")
print("Hello,", name)
```

---

### `format(template, ...values) → str`

Interpolates positional `{}` placeholders in `template` with successive `values`.

| Argument | Type | Description |
|----------|------|-------------|
| `template` | str | Format string containing `{}` placeholders |
| `...values` | any (variadic) | Values substituted left-to-right for each `{}` |

**Returns:** `str` — the formatted string.

```unilang
msg = format("{} + {} = {}", 1, 2, 3)
print(msg)   # 1 + 2 = 3
```

---

## 2. Type Conversion

### `int(value) → int`

Converts `value` to an integer. Truncates floats, parses strings, maps `True` → `1` and `False` → `0`.

| Argument | Type | Description |
|----------|------|-------------|
| `value` | int \| float \| bool \| str | Value to convert |

**Returns:** `int`

```unilang
x = int("42")      # 42
y = int(3.9)       # 3
z = int(True)      # 1
```

---

### `float(value) → float`

Converts `value` to a floating-point number.

| Argument | Type | Description |
|----------|------|-------------|
| `value` | int \| float \| bool \| str | Value to convert |

**Returns:** `float`

```unilang
x = float("3.14")  # 3.14
y = float(7)       # 7.0
```

---

### `str(value) → str`

Converts any `value` to its string representation.

| Argument | Type | Description |
|----------|------|-------------|
| `value` | any | Value to stringify |

**Returns:** `str`

```unilang
s = str(123)       # "123"
s = str(True)      # "true"
```

---

### `bool(value) → bool`

Returns the truthiness of `value`. Follows standard falsy rules (0, `""`, empty list/dict, `None` are falsy).

| Argument | Type | Description |
|----------|------|-------------|
| `value` | any | Value to evaluate |

**Returns:** `bool`

```unilang
b = bool(0)        # False
b = bool("hello")  # True
```

---

## 3. Type Checking

### `type(value) → str`

Returns the type name of `value` as a string.

| Argument | Type | Description |
|----------|------|-------------|
| `value` | any | Value to inspect |

**Returns:** `str` — one of `"int"`, `"float"`, `"str"`, `"bool"`, `"NoneType"`, `"list"`, `"dict"`, `"function"`, `"builtin_function"`, `"type"`, or the class name for instances.

Also available as `type_of(value)`.

```unilang
print(type(42))       # int
print(type([1, 2]))   # list
```

---

### `isinstance(value, type_name) → bool`

Checks whether `value`'s type matches `type_name`.

| Argument | Type | Description |
|----------|------|-------------|
| `value` | any | Value to test |
| `type_name` | str | Expected type name (same values as `type()` returns) |

**Returns:** `bool`

```unilang
print(isinstance(3.14, "float"))  # True
print(isinstance("hi", "int"))    # False
```

---

## 4. Utility

### `hash(value) → int`

Returns a 64-bit integer hash of `value`. Supports `int`, `str`, `bool`, and `None`.

| Argument | Type | Description |
|----------|------|-------------|
| `value` | int \| str \| bool \| None | Hashable value |

**Returns:** `int`

```unilang
h = hash("hello")
print(h)   # some integer
```

---

### `id(value) → int`

Returns a stable integer identity for `value`. For `int` and `bool` this is the numeric value itself; all other types return `0`.

| Argument | Type | Description |
|----------|------|-------------|
| `value` | any | Value to identify |

**Returns:** `int`

```unilang
print(id(True))   # 1
print(id(42))     # 42
```

---

## 5. Aggregates

### `sum(list) → int | float`

Sums all numeric elements of `list`. Returns `int` if all elements are integers, otherwise `float`.

| Argument | Type | Description |
|----------|------|-------------|
| `list` | list[int \| float] | List of numbers to sum |

**Returns:** `int` or `float`

```unilang
total = sum([1, 2, 3, 4])   # 10
total = sum([1.5, 2.5])     # 4.0
```

---

### `any(list) → bool`

Returns `True` if at least one element of `list` is truthy.

| Argument | Type | Description |
|----------|------|-------------|
| `list` | list | List of values to test |

**Returns:** `bool`

```unilang
print(any([False, 0, "hi"]))  # True
print(any([False, 0, None]))  # False
```

---

### `all(list) → bool`

Returns `True` if every element of `list` is truthy.

| Argument | Type | Description |
|----------|------|-------------|
| `list` | list | List of values to test |

**Returns:** `bool`

```unilang
print(all([1, True, "ok"]))  # True
print(all([1, 0, "ok"]))     # False
```

---

## 6. Character Conversion

### `chr(code) → str`

Returns the single character corresponding to the Unicode code point `code`.

| Argument | Type | Description |
|----------|------|-------------|
| `code` | int | Unicode code point (0–1114111) |

**Returns:** `str` — a one-character string.

```unilang
print(chr(65))   # A
print(chr(9829)) # ♥
```

---

### `ord(char) → int`

Returns the Unicode code point of the single character `char`.

| Argument | Type | Description |
|----------|------|-------------|
| `char` | str | A single-character string |

**Returns:** `int`

```unilang
print(ord("A"))  # 65
print(ord("♥"))  # 9829
```

---

## 7. Collection Constructors

### `list(value?) → list`

Creates a new list. With no argument returns `[]`. Converts a string into a list of single-character strings, or a dict into its list of keys.

| Argument | Type | Description |
|----------|------|-------------|
| `value` | list \| str \| dict (optional) | Source to convert; omit for empty list |

**Returns:** `list`

```unilang
chars = list("abc")     # ["a", "b", "c"]
empty = list()          # []
ks    = list({"a": 1})  # ["a"]
```

---

### `dict(value?) → dict`

Creates a new dict. With no argument returns `{}`. Accepts an existing dict or a list of `[key, value]` pairs.

| Argument | Type | Description |
|----------|------|-------------|
| `value` | dict \| list (optional) | Existing dict or list of `[key, value]` pairs; omit for empty dict |

**Returns:** `dict`

```unilang
d = dict([["x", 1], ["y", 2]])  # {"x": 1, "y": 2}
e = dict()                       # {}
```

---

## 8. Collections

### `len(value) → int`

Returns the number of elements in a string, list, or dict.

| Argument | Type | Description |
|----------|------|-------------|
| `value` | str \| list \| dict | Collection or string to measure |

**Returns:** `int`

```unilang
print(len("hello"))       # 5
print(len([1, 2, 3]))     # 3
print(len({"a": 1}))      # 1
```

---

### `range(stop) → list`
### `range(start, stop) → list`
### `range(start, stop, step) → list`

Generates a list of integers from `start` (default `0`) up to but not including `stop`, advancing by `step` (default `1`). Step may be negative for descending ranges.

| Argument | Type | Description |
|----------|------|-------------|
| `start` | int (optional) | Start value, inclusive (default `0`) |
| `stop` | int | End value, exclusive |
| `step` | int (optional) | Increment per step, must not be `0` (default `1`) |

**Returns:** `list[int]`

```unilang
r = range(5)         # [0, 1, 2, 3, 4]
r = range(2, 7)      # [2, 3, 4, 5, 6]
r = range(10, 0, -3) # [10, 7, 4, 1]
```

---

### `sorted(list) → list`

Returns a new list with all elements in ascending order. The original list is not modified.

| Argument | Type | Description |
|----------|------|-------------|
| `list` | list | List to sort |

**Returns:** `list`

```unilang
s = sorted([3, 1, 4, 1, 5])  # [1, 1, 3, 4, 5]
```

---

### `reversed(list) → list`

Returns a new list with elements in reversed order. The original list is not modified.

| Argument | Type | Description |
|----------|------|-------------|
| `list` | list | List to reverse |

**Returns:** `list`

```unilang
r = reversed([1, 2, 3])  # [3, 2, 1]
```

---

### `enumerate(list) → list`

Wraps each element of `list` into a `[index, element]` pair, producing a list of pairs.

| Argument | Type | Description |
|----------|------|-------------|
| `list` | list | List to enumerate |

**Returns:** `list[list]` — each inner list is `[int, value]`.

```unilang
for pair in enumerate(["a", "b", "c"]) {
    print(pair[0], pair[1])   # 0 a / 1 b / 2 c
}
```

---

### `zip(list1, list2) → list`

Combines two lists element-by-element into a list of `[a, b]` pairs. Truncates to the shorter list.

| Argument | Type | Description |
|----------|------|-------------|
| `list1` | list | First list |
| `list2` | list | Second list |

**Returns:** `list[list]` — each inner list is `[value_from_list1, value_from_list2]`.

```unilang
pairs = zip([1, 2, 3], ["a", "b", "c"])
# [[1, "a"], [2, "b"], [3, "c"]]
```

---

### `append(list, item) → list`

Returns a new list with `item` appended to the end. The original list is not modified.

| Argument | Type | Description |
|----------|------|-------------|
| `list` | list | Source list |
| `item` | any | Value to append |

**Returns:** `list`

```unilang
nums = append([1, 2], 3)  # [1, 2, 3]
```

---

### `keys(dict) → list`

Returns a list of all keys in `dict` in insertion order.

| Argument | Type | Description |
|----------|------|-------------|
| `dict` | dict | Dictionary to inspect |

**Returns:** `list`

```unilang
d = {"x": 1, "y": 2}
print(keys(d))   # ["x", "y"]
```

---

### `values(dict) → list`

Returns a list of all values in `dict` in insertion order.

| Argument | Type | Description |
|----------|------|-------------|
| `dict` | dict | Dictionary to inspect |

**Returns:** `list`

```unilang
d = {"x": 1, "y": 2}
print(values(d))   # [1, 2]
```

---

### `has_key(dict, key) → bool`

Returns `True` if `key` exists in `dict`.

| Argument | Type | Description |
|----------|------|-------------|
| `dict` | dict | Dictionary to search |
| `key` | any | Key to look for |

**Returns:** `bool`

```unilang
d = {"name": "Alice"}
print(has_key(d, "name"))   # True
print(has_key(d, "age"))    # False
```

---

## 9. String

### `upper(s) → str`

Converts all characters in `s` to uppercase.

| Argument | Type | Description |
|----------|------|-------------|
| `s` | str | Input string |

**Returns:** `str`

```unilang
print(upper("hello"))  # HELLO
```

---

### `lower(s) → str`

Converts all characters in `s` to lowercase.

| Argument | Type | Description |
|----------|------|-------------|
| `s` | str | Input string |

**Returns:** `str`

```unilang
print(lower("HELLO"))  # hello
```

---

### `split(s, sep?) → list`

Splits `s` by `sep` (default `" "`), returning a list of substrings.

| Argument | Type | Description |
|----------|------|-------------|
| `s` | str | String to split |
| `sep` | str (optional) | Separator (default single space `" "`) |

**Returns:** `list[str]`

```unilang
parts = split("a,b,c", ",")  # ["a", "b", "c"]
words = split("hello world") # ["hello", "world"]
```

---

### `join(list, sep) → str`

Joins all elements of `list` into a single string separated by `sep`. Both argument orderings are accepted: `join(list, sep)` or `join(sep, list)`.

| Argument | Type | Description |
|----------|------|-------------|
| `list` | list | Elements to join (each converted to string) |
| `sep` | str | Separator placed between elements |

**Returns:** `str`

```unilang
s = join(["a", "b", "c"], "-")  # "a-b-c"
s = join(", ", [1, 2, 3])        # "1, 2, 3"
```

---

### `strip(s) → str`

Removes leading and trailing whitespace from `s`.

| Argument | Type | Description |
|----------|------|-------------|
| `s` | str | String to trim |

**Returns:** `str`

```unilang
print(strip("  hello  "))  # "hello"
```

---

### `replace(s, old, new) → str`

Replaces all occurrences of `old` with `new` inside `s`.

| Argument | Type | Description |
|----------|------|-------------|
| `s` | str | Source string |
| `old` | str | Substring to search for |
| `new` | str | Replacement substring |

**Returns:** `str`

```unilang
result = replace("aabbcc", "b", "X")  # "aaXXcc"
```

---

### `contains(s, substr) → bool`

Returns `True` if `s` contains `substr`.

| Argument | Type | Description |
|----------|------|-------------|
| `s` | str | String to search in |
| `substr` | str | Substring to search for |

**Returns:** `bool`

```unilang
print(contains("foobar", "oba"))  # True
```

---

### `startswith(s, prefix) → bool`

Returns `True` if `s` starts with `prefix`. Also available as `starts_with`.

| Argument | Type | Description |
|----------|------|-------------|
| `s` | str | String to test |
| `prefix` | str | Expected prefix |

**Returns:** `bool`

```unilang
print(startswith("hello", "hel"))  # True
```

---

### `endswith(s, suffix) → bool`

Returns `True` if `s` ends with `suffix`. Also available as `ends_with`.

| Argument | Type | Description |
|----------|------|-------------|
| `s` | str | String to test |
| `suffix` | str | Expected suffix |

**Returns:** `bool`

```unilang
print(endswith("hello", "llo"))  # True
```

---

## 10. Math

### `abs(x) → int | float`

Returns the absolute value of `x`. Preserves the input type.

| Argument | Type | Description |
|----------|------|-------------|
| `x` | int \| float | Numeric value |

**Returns:** `int` or `float`

```unilang
print(abs(-7))    # 7
print(abs(-3.5))  # 3.5
```

---

### `min(...) → int | float | str`

Returns the minimum value. Accepts either a single list or two or more separate values.

| Argument | Type | Description |
|----------|------|-------------|
| `list` | list | Single list of comparable values, **or** |
| `a, b, ...` | any (variadic) | Two or more comparable values |

**Returns:** the smallest element (same type as input).

```unilang
print(min([3, 1, 4]))  # 1
print(min(5, 2, 8))    # 2
```

---

### `max(...) → int | float | str`

Returns the maximum value. Accepts either a single list or two or more separate values.

| Argument | Type | Description |
|----------|------|-------------|
| `list` | list | Single list of comparable values, **or** |
| `a, b, ...` | any (variadic) | Two or more comparable values |

**Returns:** the largest element (same type as input).

```unilang
print(max([3, 1, 4]))  # 4
print(max(5, 2, 8))    # 8
```

---

### `pow(base, exp) → int | float`

Raises `base` to the power `exp`. Returns `int` when both arguments are integers and `exp >= 0`; returns `float` otherwise.

| Argument | Type | Description |
|----------|------|-------------|
| `base` | int \| float | Base value |
| `exp` | int \| float | Exponent |

**Returns:** `int` or `float`

```unilang
print(pow(2, 10))    # 1024
print(pow(2, -1))    # 0.5
print(pow(2.0, 0.5)) # 1.4142...
```

---

### `sqrt(x) → float`

Returns the square root of `x`. Raises an error if `x < 0`.

| Argument | Type | Description |
|----------|------|-------------|
| `x` | int \| float | Non-negative number |

**Returns:** `float`

```unilang
print(sqrt(9))    # 3.0
print(sqrt(2.0))  # 1.4142...
```

---

### `floor(x) → int`

Returns the largest integer less than or equal to `x`.

| Argument | Type | Description |
|----------|------|-------------|
| `x` | int \| float | Numeric value |

**Returns:** `int`

```unilang
print(floor(3.9))   # 3
print(floor(-3.1))  # -4
```

---

### `ceil(x) → int`

Returns the smallest integer greater than or equal to `x`.

| Argument | Type | Description |
|----------|------|-------------|
| `x` | int \| float | Numeric value |

**Returns:** `int`

```unilang
print(ceil(3.1))   # 4
print(ceil(-3.9))  # -3
```

---

### `round(x) → int`

Rounds `x` to the nearest integer (half-away from zero).

| Argument | Type | Description |
|----------|------|-------------|
| `x` | int \| float | Numeric value |

**Returns:** `int`

```unilang
print(round(3.5))   # 4
print(round(-2.5))  # -3
```

---

### `log(x) → float`
### `log(x, base) → float`

Returns the natural logarithm of `x`, or the logarithm in the given `base` when provided. `x` must be positive; `base` must be positive and not `1`.

| Argument | Type | Description |
|----------|------|-------------|
| `x` | int \| float | Positive number |
| `base` | int \| float (optional) | Logarithm base (default: natural log) |

**Returns:** `float`

```unilang
print(log(E))        # 1.0
print(log(100, 10))  # 2.0
```

---

### `log2(x) → float`

Returns the base-2 logarithm of `x`. `x` must be positive.

| Argument | Type | Description |
|----------|------|-------------|
| `x` | int \| float | Positive number |

**Returns:** `float`

```unilang
print(log2(8))   # 3.0
```

---

### `log10(x) → float`

Returns the base-10 logarithm of `x`. `x` must be positive.

| Argument | Type | Description |
|----------|------|-------------|
| `x` | int \| float | Positive number |

**Returns:** `float`

```unilang
print(log10(1000))  # 3.0
```

---

### `sin(x) → float`

Returns the sine of `x` (in radians).

| Argument | Type | Description |
|----------|------|-------------|
| `x` | int \| float | Angle in radians |

**Returns:** `float`

```unilang
print(sin(PI / 2))  # 1.0
```

---

### `cos(x) → float`

Returns the cosine of `x` (in radians).

| Argument | Type | Description |
|----------|------|-------------|
| `x` | int \| float | Angle in radians |

**Returns:** `float`

```unilang
print(cos(0))  # 1.0
```

---

### `tan(x) → float`

Returns the tangent of `x` (in radians).

| Argument | Type | Description |
|----------|------|-------------|
| `x` | int \| float | Angle in radians |

**Returns:** `float`

```unilang
print(tan(PI / 4))  # ~1.0
```

---

### `asin(x) → float`

Returns the arcsine of `x` in radians. `x` must be in `[-1, 1]`.

| Argument | Type | Description |
|----------|------|-------------|
| `x` | int \| float | Value in `[-1.0, 1.0]` |

**Returns:** `float` — result in `[-π/2, π/2]`.

```unilang
print(asin(1.0))  # ~1.5708 (π/2)
```

---

### `acos(x) → float`

Returns the arccosine of `x` in radians. `x` must be in `[-1, 1]`.

| Argument | Type | Description |
|----------|------|-------------|
| `x` | int \| float | Value in `[-1.0, 1.0]` |

**Returns:** `float` — result in `[0, π]`.

```unilang
print(acos(1.0))  # 0.0
```

---

### `atan(x) → float`

Returns the arctangent of `x` in radians.

| Argument | Type | Description |
|----------|------|-------------|
| `x` | int \| float | Numeric value |

**Returns:** `float` — result in `(-π/2, π/2)`.

```unilang
print(atan(1.0))  # ~0.7854 (π/4)
```

---

### `atan2(y, x) → float`

Returns the angle in radians between the positive x-axis and the point `(x, y)`, in the range `(-π, π]`.

| Argument | Type | Description |
|----------|------|-------------|
| `y` | int \| float | Y component |
| `x` | int \| float | X component |

**Returns:** `float`

```unilang
print(atan2(1.0, 1.0))  # ~0.7854 (π/4)
```

---

### `exp(x) → float`

Returns e raised to the power `x`.

| Argument | Type | Description |
|----------|------|-------------|
| `x` | int \| float | Exponent |

**Returns:** `float`

```unilang
print(exp(1))  # ~2.71828 (E)
```

---

### `hypot(a, b) → float`

Returns the Euclidean distance `sqrt(a² + b²)`.

| Argument | Type | Description |
|----------|------|-------------|
| `a` | int \| float | First leg |
| `b` | int \| float | Second leg |

**Returns:** `float`

```unilang
print(hypot(3, 4))  # 5.0
```

---

### `gcd(a, b) → int`

Returns the greatest common divisor of integers `a` and `b`.

| Argument | Type | Description |
|----------|------|-------------|
| `a` | int | First integer |
| `b` | int | Second integer |

**Returns:** `int`

```unilang
print(gcd(48, 18))  # 6
```

---

### `factorial(n) → int`

Returns `n!`. `n` must be a non-negative integer no greater than `20`.

| Argument | Type | Description |
|----------|------|-------------|
| `n` | int | Non-negative integer, `0 ≤ n ≤ 20` |

**Returns:** `int`

```unilang
print(factorial(5))  # 120
```

---

### `clamp(value, min, max) → int | float`

Clamps `value` into the inclusive range `[min, max]`.

| Argument | Type | Description |
|----------|------|-------------|
| `value` | int \| float | Value to clamp |
| `min` | int \| float | Lower bound (inclusive) |
| `max` | int \| float | Upper bound (inclusive) |

**Returns:** `int` when all three are integers, otherwise `float`.

```unilang
print(clamp(15, 0, 10))   # 10
print(clamp(-5, 0, 10))   # 0
print(clamp(7, 0, 10))    # 7
```

---

## 10a. Math Constants

The following global constants are pre-set by the math module:

| Constant | Value | Description |
|----------|-------|-------------|
| `PI` | `3.141592653589793` | The mathematical constant π |
| `E` | `2.718281828459045` | Euler's number e |

```unilang
area = PI * pow(r, 2)
```

---

## 11. JSON

### `json_encode(value) → str`

Serializes `value` to a JSON string. Also available as `to_json`.

| Argument | Type | Description |
|----------|------|-------------|
| `value` | any | Value to serialize (None, bool, int, float, str, list, dict) |

**Returns:** `str` — valid JSON text. `NaN` and infinite floats are encoded as `null`.

```unilang
s = json_encode({"name": "Alice", "age": 30})
# '{"name":"Alice","age":30}'
```

---

### `json_decode(s) → any`

Parses a JSON string and returns the corresponding UniLang value. Also available as `from_json`.

| Argument | Type | Description |
|----------|------|-------------|
| `s` | str | Valid JSON string |

**Returns:** `None | bool | int | float | str | list | dict`

```unilang
obj = json_decode('{"x": 1, "y": [2, 3]}')
print(obj["x"])  # 1
```

---

## 12. Time

### `now() → int`

Returns the current Unix time in **milliseconds** since the epoch.

**Returns:** `int`

```unilang
ms = now()
print(ms)   # e.g. 1713340800000
```

---

### `sleep(seconds) → None`

Pauses execution for the given number of seconds.

| Argument | Type | Description |
|----------|------|-------------|
| `seconds` | int \| float | Duration to sleep (negative or zero is a no-op) |

**Returns:** `None`

```unilang
sleep(1.5)   # wait 1.5 seconds
```

---

## 13. DateTime

DateTime functions work with a **datetime dict** — a `dict` with the following keys:

| Key | Type | Description |
|-----|------|-------------|
| `year` | int | Calendar year |
| `month` | int | Month (1–12) |
| `day` | int | Day of month (1–31) |
| `hour` | int | Hour (0–23) |
| `minute` | int | Minute (0–59) |
| `second` | int | Second (0–59) |
| `microsecond` | int | Microseconds (0–999999) |
| `timestamp` | float | Unix timestamp (seconds since epoch) |

---

### `datetime_now() → dict`

Returns the current local date and time as a datetime dict.

**Returns:** `dict` (datetime dict)

```unilang
dt = datetime_now()
print(dt["year"], dt["month"], dt["day"])
```

---

### `datetime_utcnow() → dict`

Returns the current UTC date and time as a datetime dict.

**Returns:** `dict` (datetime dict)

```unilang
utc = datetime_utcnow()
print(utc["hour"])
```

---

### `datetime_parse(s, fmt) → dict | None`

Parses a date/time string `s` using the `strftime`-compatible format `fmt`. Returns `None` if parsing fails.

| Argument | Type | Description |
|----------|------|-------------|
| `s` | str | Date/time string to parse |
| `fmt` | str | `strftime`-compatible format string (e.g. `"%Y-%m-%d %H:%M:%S"`) |

**Returns:** `dict` (datetime dict) or `None`

```unilang
dt = datetime_parse("2024-06-15 09:30:00", "%Y-%m-%d %H:%M:%S")
print(dt["year"])  # 2024
```

---

### `datetime_format(dt_dict, fmt) → str`

Formats a datetime dict as a string using the `strftime`-compatible format `fmt`.

| Argument | Type | Description |
|----------|------|-------------|
| `dt_dict` | dict | Datetime dict |
| `fmt` | str | `strftime`-compatible format string |

**Returns:** `str`

```unilang
dt = datetime_now()
s = datetime_format(dt, "%Y-%m-%d")
print(s)   # e.g. "2024-06-15"
```

---

### `datetime_add(dt_dict, delta_dict) → dict`

Adds a duration to a datetime. `delta_dict` may contain any subset of the keys `days`, `hours`, `minutes`, `seconds` (all `int`).

| Argument | Type | Description |
|----------|------|-------------|
| `dt_dict` | dict | Datetime dict to add to |
| `delta_dict` | dict | Duration dict with optional keys: `days`, `hours`, `minutes`, `seconds` |

**Returns:** `dict` (datetime dict)

```unilang
dt    = datetime_now()
later = datetime_add(dt, {"hours": 2, "minutes": 30})
```

---

### `datetime_diff_seconds(dt1, dt2) → float`

Returns the signed difference `dt1 - dt2` in seconds.

| Argument | Type | Description |
|----------|------|-------------|
| `dt1` | dict | Datetime dict (minuend) |
| `dt2` | dict | Datetime dict (subtrahend) |

**Returns:** `float` — positive if `dt1` is later than `dt2`.

```unilang
diff = datetime_diff_seconds(end_dt, start_dt)
print(diff)  # elapsed seconds
```

---

### `timestamp_to_datetime(ts) → dict`

Converts a Unix timestamp (float seconds) to a UTC datetime dict.

| Argument | Type | Description |
|----------|------|-------------|
| `ts` | int \| float | Unix timestamp in seconds |

**Returns:** `dict` (datetime dict)

```unilang
dt = timestamp_to_datetime(1718438400.0)
print(dt["year"])  # 2024
```

---

### `datetime_to_timestamp(dt_dict) → float`

Converts a datetime dict to a Unix timestamp (treating the dict as UTC).

| Argument | Type | Description |
|----------|------|-------------|
| `dt_dict` | dict | Datetime dict |

**Returns:** `float` — Unix timestamp in seconds.

```unilang
ts = datetime_to_timestamp(datetime_utcnow())
```

---

## 14. File

### `read_file(path) → str`

Reads the entire contents of the file at `path` and returns it as a string.

| Argument | Type | Description |
|----------|------|-------------|
| `path` | str | File path to read |

**Returns:** `str`

```unilang
content = read_file("/etc/hostname")
print(content)
```

---

### `write_file(path, content) → bool`

Writes `content` to the file at `path`, creating or overwriting it.

| Argument | Type | Description |
|----------|------|-------------|
| `path` | str | Destination file path |
| `content` | str \| any | Content to write (non-strings are converted via `str()`) |

**Returns:** `True` on success.

```unilang
write_file("/tmp/out.txt", "Hello, file!")
```

---

### `file_exists(path) → bool`

Returns `True` if a file or directory exists at `path`.

| Argument | Type | Description |
|----------|------|-------------|
| `path` | str | Path to test |

**Returns:** `bool`

```unilang
if file_exists("/tmp/data.csv") {
    data = read_file("/tmp/data.csv")
}
```

---

### `file_size(path) → int`

Returns the size of the file at `path` in bytes.

| Argument | Type | Description |
|----------|------|-------------|
| `path` | str | File path |

**Returns:** `int` — file size in bytes.

```unilang
sz = file_size("/tmp/out.txt")
print(sz, "bytes")
```

---

### `list_dir(path) → list`

Returns a list of entry names (files and subdirectories) in the directory at `path`.

| Argument | Type | Description |
|----------|------|-------------|
| `path` | str | Directory path |

**Returns:** `list[str]`

```unilang
entries = list_dir("/tmp")
for name in entries {
    print(name)
}
```

---

## 15. Environment

### `env_get(name) → str | None`

Reads the value of the environment variable `name`. Returns `None` if the variable is not set.

| Argument | Type | Description |
|----------|------|-------------|
| `name` | str | Name of the environment variable |

**Returns:** `str` or `None`

```unilang
home = env_get("HOME")
print(home)   # e.g. "/Users/alice"
```

---

### `env_set(name, value) → bool`

Sets the environment variable `name` to `value` for the current process. Non-string values are converted to strings.

| Argument | Type | Description |
|----------|------|-------------|
| `name` | str | Name of the environment variable |
| `value` | str \| any | Value to set (non-strings are stringified) |

**Returns:** `True`

```unilang
env_set("DEBUG", "1")
```

---

## 16. HTTP Client

All HTTP client functions return a **response dict** with the following keys:

| Key | Type | Description |
|-----|------|-------------|
| `status` | int | HTTP status code (e.g. `200`, `404`) |
| `body` | str | Response body as a string |
| `ok` | bool | `True` when `status < 400` |

---

### `http_get(url) → dict`

Performs an HTTP GET request.

| Argument | Type | Description |
|----------|------|-------------|
| `url` | str | Target URL |

**Returns:** response dict

```unilang
res = http_get("https://api.example.com/users")
if res["ok"] {
    data = json_decode(res["body"])
}
```

---

### `http_post(url, body) → dict`

Performs an HTTP POST request. `Content-Type` is automatically set to `application/json` when `body` starts with `{` or `[`, otherwise `text/plain`.

| Argument | Type | Description |
|----------|------|-------------|
| `url` | str | Target URL |
| `body` | str \| any | Request body (non-strings are stringified) |

**Returns:** response dict

```unilang
payload = json_encode({"name": "Bob"})
res = http_post("https://api.example.com/users", payload)
```

---

### `http_put(url, body) → dict`

Performs an HTTP PUT request. Content-Type detection follows the same rule as `http_post`.

| Argument | Type | Description |
|----------|------|-------------|
| `url` | str | Target URL |
| `body` | str \| any | Request body |

**Returns:** response dict

```unilang
payload = json_encode({"name": "Alice"})
res = http_put("https://api.example.com/users/1", payload)
```

---

### `http_delete(url) → dict`

Performs an HTTP DELETE request.

| Argument | Type | Description |
|----------|------|-------------|
| `url` | str | Target URL |

**Returns:** response dict

```unilang
res = http_delete("https://api.example.com/users/42")
print(res["status"])
```

---

## 17. HTTP Server

### `serve(port, handler) → None`

Starts a blocking HTTP server on the given `port`. `handler` is called for every incoming request and must return a response dict.

> **Note:** `serve` is registered as a native function value (not a normal builtin) because it requires mutable VM access for routing. Call it like any other function.

| Argument | Type | Description |
|----------|------|-------------|
| `port` | int | TCP port to listen on |
| `handler` | function | Request handler `fn(request_dict) → response_dict` |

The `request_dict` passed to the handler contains at minimum:

| Key | Type | Description |
|-----|------|-------------|
| `method` | str | HTTP method (`"GET"`, `"POST"`, etc.) |
| `path` | str | Request path (e.g. `"/api/users"`) |
| `body` | str | Request body (may be empty) |

The handler should return a dict with:

| Key | Type | Description |
|-----|------|-------------|
| `status` | int | HTTP status code |
| `body` | str | Response body |

```unilang
fn handle(req) {
    return {"status": 200, "body": "Hello!"}
}
serve(8080, handle)
```

---

## 18. Random

### `random() → float`

Returns a pseudo-random float in `[0.0, 1.0)` using an xorshift64 generator seeded from the system clock nanoseconds.

**Returns:** `float`

```unilang
x = random()
print(x)   # e.g. 0.7341823...
```

---

### `random_int(min, max) → int`

Returns a pseudo-random integer in the inclusive range `[min, max]`.

| Argument | Type | Description |
|----------|------|-------------|
| `min` | int | Lower bound (inclusive) |
| `max` | int | Upper bound (inclusive), must be `>= min` |

**Returns:** `int`

```unilang
dice = random_int(1, 6)
print(dice)   # 1 to 6
```

---

## 19. Regex

All regex functions take a pattern string that follows standard RE2/PCRE syntax as implemented by the Rust `regex` crate (no backtracking, no look-around).

### `regex_match(pattern, text) → bool`

Returns `True` if `pattern` matches anywhere within `text`.

| Argument | Type | Description |
|----------|------|-------------|
| `pattern` | str | Regular expression pattern |
| `text` | str | Text to search |

**Returns:** `bool`

```unilang
print(regex_match("\\d+", "abc123"))  # True
```

---

### `regex_match_full(pattern, text) → bool`

Returns `True` if `pattern` matches the **entire** `text` (implicitly anchored with `^...$`).

| Argument | Type | Description |
|----------|------|-------------|
| `pattern` | str | Regular expression pattern |
| `text` | str | Text to test |

**Returns:** `bool`

```unilang
print(regex_match_full("\\d+", "123"))    # True
print(regex_match_full("\\d+", "abc123")) # False
```

---

### `regex_find(pattern, text) → str | None`

Returns the first substring of `text` that matches `pattern`, or `None` if there is no match.

| Argument | Type | Description |
|----------|------|-------------|
| `pattern` | str | Regular expression pattern |
| `text` | str | Text to search |

**Returns:** `str` or `None`

```unilang
m = regex_find("\\d+", "abc 42 xyz")
print(m)   # "42"
```

---

### `regex_find_all(pattern, text) → list`

Returns a list of all non-overlapping substrings of `text` that match `pattern`.

| Argument | Type | Description |
|----------|------|-------------|
| `pattern` | str | Regular expression pattern |
| `text` | str | Text to search |

**Returns:** `list[str]`

```unilang
nums = regex_find_all("\\d+", "a1 b22 c333")
# ["1", "22", "333"]
```

---

### `regex_replace(pattern, text, replacement) → str`

Replaces the **first** match of `pattern` in `text` with `replacement`.

| Argument | Type | Description |
|----------|------|-------------|
| `pattern` | str | Regular expression pattern |
| `text` | str | Text to modify |
| `replacement` | str | Replacement string (supports `$1` capture group references) |

**Returns:** `str`

```unilang
result = regex_replace("\\d+", "abc 42 def 7", "NUM")
# "abc NUM def 7"
```

---

### `regex_replace_all(pattern, text, replacement) → str`

Replaces **all** matches of `pattern` in `text` with `replacement`.

| Argument | Type | Description |
|----------|------|-------------|
| `pattern` | str | Regular expression pattern |
| `text` | str | Text to modify |
| `replacement` | str | Replacement string (supports `$1` capture group references) |

**Returns:** `str`

```unilang
result = regex_replace_all("\\d+", "abc 42 def 7", "NUM")
# "abc NUM def NUM"
```

---

### `regex_split(pattern, text) → list`

Splits `text` at every match of `pattern`, returning the list of substrings between matches.

| Argument | Type | Description |
|----------|------|-------------|
| `pattern` | str | Regular expression pattern |
| `text` | str | Text to split |

**Returns:** `list[str]`

```unilang
parts = regex_split("\\s+", "hello   world  foo")
# ["hello", "world", "foo"]
```

---

### `regex_groups(pattern, text) → list | None`

Returns the capture groups from the **first** match of `pattern` in `text`. Group 0 (the full match) is excluded. Returns `None` if there is no match.

| Argument | Type | Description |
|----------|------|-------------|
| `pattern` | str | Regular expression with capture groups |
| `text` | str | Text to search |

**Returns:** `list[str | None]` or `None`. Each list element corresponds to a capture group; unmatched optional groups are `None`.

```unilang
groups = regex_groups("(\\w+)@(\\w+\\.\\w+)", "user@example.com")
# ["user", "example.com"]
```

---

## 20. UUID

### `uuid_v4() → str`

Generates a random version-4 UUID.

**Returns:** `str` — hyphenated lowercase UUID, e.g. `"550e8400-e29b-41d4-a716-446655440000"`.

```unilang
id = uuid_v4()
print(id)
```

---

### `uuid_is_valid(s) → bool`

Returns `True` if `s` is a valid UUID in any standard format.

| Argument | Type | Description |
|----------|------|-------------|
| `s` | str | String to validate |

**Returns:** `bool`

```unilang
print(uuid_is_valid("550e8400-e29b-41d4-a716-446655440000"))  # True
print(uuid_is_valid("not-a-uuid"))                            # False
```

---

### `uuid_parse(s) → str`

Parses `s` as a UUID and returns the canonical hyphenated lowercase representation. Raises an error if `s` is not a valid UUID.

| Argument | Type | Description |
|----------|------|-------------|
| `s` | str | UUID string (any standard format) |

**Returns:** `str` — canonical hyphenated UUID.

```unilang
canonical = uuid_parse("550E8400E29B41D4A716446655440000")
print(canonical)  # "550e8400-e29b-41d4-a716-446655440000"
```

---

## 21. Base64

### `base64_encode(s) → str`

Encodes `s` using standard Base64 (RFC 4648, with `=` padding).

| Argument | Type | Description |
|----------|------|-------------|
| `s` | str | UTF-8 string to encode |

**Returns:** `str` — Base64-encoded string.

```unilang
enc = base64_encode("Hello, World!")
print(enc)  # "SGVsbG8sIFdvcmxkIQ=="
```

---

### `base64_decode(s) → str`

Decodes a standard Base64-encoded string `s` back to UTF-8.

| Argument | Type | Description |
|----------|------|-------------|
| `s` | str | Base64-encoded string |

**Returns:** `str` — decoded UTF-8 string. Raises an error if `s` is not valid Base64 or the decoded bytes are not valid UTF-8.

```unilang
dec = base64_decode("SGVsbG8sIFdvcmxkIQ==")
print(dec)  # "Hello, World!"
```

---

### `base64_encode_url(s) → str`

Encodes `s` using URL-safe Base64 (RFC 4648 §5, `-` and `_` instead of `+` and `/`, no padding).

| Argument | Type | Description |
|----------|------|-------------|
| `s` | str | UTF-8 string to encode |

**Returns:** `str` — URL-safe Base64 string without `=` padding.

```unilang
enc = base64_encode_url("Hello, World!")
print(enc)  # "SGVsbG8sIFdvcmxkIQ"
```

---

### `base64_decode_url(s) → str`

Decodes a URL-safe Base64-encoded string (no padding expected).

| Argument | Type | Description |
|----------|------|-------------|
| `s` | str | URL-safe Base64 string |

**Returns:** `str` — decoded UTF-8 string.

```unilang
dec = base64_decode_url("SGVsbG8sIFdvcmxkIQ")
print(dec)  # "Hello, World!"
```

---

## 22. Crypto

### `sha256(s) → str`

Returns the SHA-256 digest of `s` as a lowercase hex string. Also available as `hash_sha256`.

| Argument | Type | Description |
|----------|------|-------------|
| `s` | str | Input string |

**Returns:** `str` — 64-character lowercase hex string.

```unilang
h = sha256("hello")
print(h)  # "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
```

---

### `sha512(s) → str`

Returns the SHA-512 digest of `s` as a lowercase hex string.

| Argument | Type | Description |
|----------|------|-------------|
| `s` | str | Input string |

**Returns:** `str` — 128-character lowercase hex string.

```unilang
h = sha512("hello")
print(h)  # long hex string...
```

---

### `md5(s) → str`

Returns the MD5 digest of `s` as a lowercase hex string.

> **Note:** MD5 is cryptographically broken. Use it only for non-security purposes such as checksums.

| Argument | Type | Description |
|----------|------|-------------|
| `s` | str | Input string |

**Returns:** `str` — 32-character lowercase hex string.

```unilang
checksum = md5("data")
```

---

### `hmac_sha256(key, message) → str`

Computes HMAC-SHA256 of `message` using `key`, returning the result as a lowercase hex string.

| Argument | Type | Description |
|----------|------|-------------|
| `key` | str | Secret key |
| `message` | str | Message to authenticate |

**Returns:** `str` — 64-character lowercase hex string.

```unilang
sig = hmac_sha256("secret", "payload")
print(sig)
```

---

## 23. CSV

### `csv_read(path) → list`

Reads a CSV file from `path` and returns it as a list of rows, where each row is a list of strings. Headers are treated as data (not stripped).

| Argument | Type | Description |
|----------|------|-------------|
| `path` | str | Path to the CSV file |

**Returns:** `list[list[str]]`

```unilang
rows = csv_read("/data/sales.csv")
for row in rows {
    print(row[0], row[1])
}
```

---

### `csv_read_header(path) → list`

Reads a CSV file treating the first row as column headers. Returns a list of dicts, one per data row, where each dict maps header name → cell value.

| Argument | Type | Description |
|----------|------|-------------|
| `path` | str | Path to the CSV file |

**Returns:** `list[dict]`

```unilang
records = csv_read_header("/data/users.csv")
for rec in records {
    print(rec["name"], rec["email"])
}
```

---

### `csv_write(path, rows) → bool`

Writes `rows` to a CSV file at `path`. Each row must be a list of values; non-strings are converted to their string representation.

| Argument | Type | Description |
|----------|------|-------------|
| `path` | str | Destination file path |
| `rows` | list[list] | List of rows, each row is a list of cell values |

**Returns:** `True` on success.

```unilang
csv_write("/tmp/out.csv", [["name","age"],["Alice","30"]])
```

---

### `csv_parse(text) → list`

Parses a CSV-formatted string and returns it as a list of rows (list of lists of strings).

| Argument | Type | Description |
|----------|------|-------------|
| `text` | str | CSV-formatted string |

**Returns:** `list[list[str]]`

```unilang
rows = csv_parse("a,b,c\n1,2,3")
print(rows)  # [["a","b","c"], ["1","2","3"]]
```

---

### `csv_stringify(rows) → str`

Converts a list of rows into a CSV-formatted string.

| Argument | Type | Description |
|----------|------|-------------|
| `rows` | list[list] | List of rows, each row is a list of cell values |

**Returns:** `str` — CSV text.

```unilang
text = csv_stringify([["x","y"],[1,2],[3,4]])
print(text)
```

---

## 24. Global Constants

The following values are pre-set as globals and can be used directly without calling any function:

| Name | Type | Value | Description |
|------|------|-------|-------------|
| `None` | NoneType | `null` | The null/absent value |
| `True` | bool | `true` | Boolean true |
| `False` | bool | `false` | Boolean false |
| `PI` | float | `3.141592653589793` | Mathematical constant π |
| `E` | float | `2.718281828459045` | Euler's number |
| `System` | instance | Java-style facade | `System.out.println(x)` works as an alias for `println(x)` |

```unilang
print(None)   # None
print(True)   # true
print(PI)     # 3.141592653589793
System.out.println("Hello from Java style!")
```
