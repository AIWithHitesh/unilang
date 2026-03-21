# UniLang Quickstart Guide

Go from zero to running code in 5 minutes. This guide assumes you have already installed UniLang. If not, see the [Installation Guide](INSTALLATION.md) first.

---

## Table of Contents

- [Step 1: Create Your First .uniL File](#step-1-create-your-first-unil-file)
- [Step 2: Run It](#step-2-run-it)
- [Step 3: Check for Errors](#step-3-check-for-errors)
- [Step 4: See the Bytecode](#step-4-see-the-bytecode)
- [Example 2: A Multi-Paradigm Calculator](#example-2-a-multi-paradigm-calculator)
- [Example 3: ML Model in UniLang](#example-3-ml-model-in-unilang)
- [Next Steps](#next-steps)

---

## Step 1: Create Your First .uniL File

Open your favorite text editor and create a file called `hello.uniL`. Paste the following code into it:

```unilang
// Hello World in UniLang — mixing Python and Java styles
print("Hello from UniLang!")

// Python-style function
def greet(name):
    return f"Welcome, {name}!"

// Java-style class
public class Calculator {
    public int add(int a, int b) {
        return a + b;
    }
}

// Use both together
message = greet("Developer")
print(message)

calc = Calculator()
result = calc.add(5, 3)
print(f"5 + 3 = {result}")
```

Notice how Python-style functions (`def greet(name):`) and Java-style classes (`public class Calculator`) coexist in the same file. This is what makes UniLang unique.

## Step 2: Run It

Open a terminal, navigate to the folder where you saved `hello.uniL`, and run:

```bash
unilang run hello.uniL
```

You should see this output:

```
Hello from UniLang!
Welcome, Developer!
5 + 3 = 8
```

## Step 3: Check for Errors

UniLang includes a built-in checker that finds problems in your code without running it. Try it:

```bash
unilang check hello.uniL
```

If there are no errors, you will see:

```
No errors found in hello.uniL
```

If there are problems, the checker will tell you exactly which line to fix.

## Step 4: See the Bytecode

Curious about what happens under the hood? You can compile your `.uniL` file to see the intermediate bytecode:

```bash
unilang compile hello.uniL
```

This produces a compiled output and shows you how UniLang translates your mixed-paradigm code into a unified intermediate representation.

---

## Example 2: A Multi-Paradigm Calculator

Now let's build something more interesting. Create a file called `calculator.uniL`:

```unilang
// A calculator that uses Python-style functions for logic
// and Java-style classes for structure

// Python-style: quick utility functions
def square(x):
    return x * x

def cube(x):
    return x * x * x

// Java-style: structured class with methods
public class ScientificCalculator {

    private double memory = 0.0;

    public double add(double a, double b) {
        return a + b;
    }

    public double subtract(double a, double b) {
        return a - b;
    }

    public double multiply(double a, double b) {
        return a * b;
    }

    public double divide(double a, double b) {
        if (b == 0) {
            print("Error: division by zero")
            return 0.0;
        }
        return a / b;
    }

    public void storeInMemory(double value) {
        this.memory = value;
    }

    public double recallMemory() {
        return this.memory;
    }
}

// Use everything together
calc = ScientificCalculator()

// Basic operations
print(f"10 + 5 = {calc.add(10, 5)}")
print(f"10 - 5 = {calc.subtract(10, 5)}")
print(f"10 * 5 = {calc.multiply(10, 5)}")
print(f"10 / 5 = {calc.divide(10, 5)}")

// Python-style functions
print(f"4 squared = {square(4)}")
print(f"3 cubed = {cube(3)}")

// Memory feature
calc.storeInMemory(42.0)
print(f"Recalled from memory: {calc.recallMemory()}")
```

Run it:

```bash
unilang run calculator.uniL
```

Expected output:

```
10 + 5 = 15.0
10 - 5 = 5.0
10 * 5 = 50.0
10 / 5 = 2.0
4 squared = 16
3 cubed = 27
Recalled from memory: 42.0
```

---

## Example 3: ML Model in UniLang

UniLang has built-in support for defining machine learning models. Create a file called `ml_demo.uniL`:

```unilang
// ML model definition in UniLang
// Combines Python-style data handling with structured model definition

import unilang.ml

// Python-style: prepare data
def load_data():
    features = [[1.0, 2.0], [3.0, 4.0], [5.0, 6.0], [7.0, 8.0]]
    labels = [0, 1, 1, 0]
    return features, labels

def split_data(features, labels, ratio=0.75):
    split_index = int(len(features) * ratio)
    train_x = features[:split_index]
    train_y = labels[:split_index]
    test_x = features[split_index:]
    test_y = labels[split_index:]
    return train_x, train_y, test_x, test_y

// Java-style: define the model as a class
public class SimpleClassifier {

    private double learningRate;
    private int epochs;

    public SimpleClassifier(double learningRate, int epochs) {
        this.learningRate = learningRate;
        this.epochs = epochs;
    }

    public void train(List<double[]> features, List<int> labels) {
        print(f"Training with learning rate {this.learningRate} for {this.epochs} epochs...")
        for (int i = 0; i < this.epochs; i++) {
            // Training logic here
            if (i % 10 == 0) {
                print(f"  Epoch {i}: loss = {1.0 / (i + 1):.4f}")
            }
        }
        print("Training complete!")
    }

    public int predict(double[] input) {
        // Prediction logic here
        return 1;
    }
}

// Put it all together
features, labels = load_data()
train_x, train_y, test_x, test_y = split_data(features, labels)

print("=== UniLang ML Demo ===")
print(f"Training samples: {len(train_x)}")
print(f"Test samples: {len(test_x)}")

model = SimpleClassifier(0.01, 50)
model.train(train_x, train_y)

// Make predictions
for sample in test_x:
    prediction = model.predict(sample)
    print(f"Input: {sample} -> Predicted: {prediction}")

print("Done!")
```

Run it:

```bash
unilang run ml_demo.uniL
```

This example demonstrates how UniLang lets you write data-processing code in Python style and model architecture in Java style, all in one file.

---

## Next Steps

Now that you have written and run your first UniLang programs, here are some things to explore:

1. **Set up your editor** for syntax highlighting and code completion:
   - [VS Code / Cursor Setup](VSCODE_SETUP.md)
   - [IntelliJ / PyCharm Setup](JETBRAINS_SETUP.md)
   - [Eclipse Setup](ECLIPSE_SETUP.md)
   - [UniLang IDE (Standalone)](IDE_SETUP.md)

2. **Read the language specification** to learn about all supported syntax:
   - See the `docs/specifications/` folder in the repository.

3. **Explore the examples** in the `examples/` folder for more real-world use cases.

4. **Join the community** by opening issues or discussions on [GitHub](https://github.com/hiteshkumar2187/unilang).
