# UniLang ML Framework — Documentation

A neural network framework built **entirely from scratch** in UniLang.
No PyTorch. No TensorFlow. Every piece — from the math to the training loop — written by hand.

## Who is this for?

Software developers who want to understand how machine learning **actually works** under the hood, not just call `model.fit()`. If you can write a `for` loop and understand basic math (addition, multiplication), you can build a neural network with this framework.

## Documentation Map

| Document | What you'll learn | Time |
|----------|------------------|------|
| [1. Core Concepts](./01_CORE_CONCEPTS.md) | What are tensors, neurons, and neural networks — explained with code, not math papers | 15 min |
| [2. Architecture](./02_ARCHITECTURE.md) | How the framework is built, how data flows, and why each component exists | 10 min |
| [3. UniNN Model](./03_UNINN_MODEL.md) | Our custom model architecture — gated residual blocks and multi-scale mixing | 10 min |
| [4. Building Your First Model](./04_BUILD_YOUR_FIRST_MODEL.md) | Step-by-step tutorial: create, train, and use a model from scratch | 20 min |
| [5. Training Deep Dive](./05_TRAINING_DEEP_DIVE.md) | How backpropagation, optimizers, and loss functions work — visually | 15 min |
| [6. Best Practices](./06_BEST_PRACTICES.md) | Practical tips for building reliable models | 10 min |
| [7. API Reference](./07_API_REFERENCE.md) | Complete reference for every class and method | Reference |
| [8. Use Cases](./08_USE_CASES.md) | What problems this framework can solve, with examples | 10 min |

## Quick Start (30 seconds)

```unilang
from models.uniNN import UniNN
from core.loss import CrossEntropyLoss
from core.optimizers import Adam

// 1. Create a model
model = UniNN(inputDim=10, hiddenDim=64, outputDim=3, task="classification")

// 2. Pick a loss function and optimizer
loss_fn = CrossEntropyLoss()
optimizer = Adam(model.parameters(), lr=0.001)

// 3. Train
for epoch in range(100):
    model.zero_grad()
    predictions = model.forward(X_train)
    loss = loss_fn.compute(predictions, y_train)
    loss.backward()
    optimizer.step()

// 4. Save and share
model.save("my_model.json")
```

That's it. No GPU required. No pip install pytorch. Just UniLang.
