# UniLang ML Framework

A neural network framework built **entirely from scratch** in UniLang — no PyTorch, no TensorFlow, no external ML libraries.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                   UniLang ML Framework                  │
├─────────────────────────────────────────────────────────┤
│  core/tensor.uniL     — Tensor with autograd engine     │
│  core/layers.uniL     — Neurons, Linear, BatchNorm,     │
│                         Dropout, Embedding, LSTM,        │
│                         Conv1D, MaxPool1D                │
│  core/network.uniL    — Sequential, ParallelEnsemble    │
│  core/loss.uniL       — MSE, CrossEntropy, BCE, Huber   │
│  core/optimizers.uniL — SGD, Adam, RMSProp + schedulers │
│  core/trainer.uniL    — Training loop with metrics      │
├─────────────────────────────────────────────────────────┤
│  models/uniNN.uniL    — UniNN: Flagship model           │
│                         Gated Residual + Multi-Scale     │
├─────────────────────────────────────────────────────────┤
│  examples/train_model.uniL      — Train from scratch    │
│  examples/inference_example.uniL — Load & predict       │
└─────────────────────────────────────────────────────────┘
```

## UniNN Model

**UniNN** (UniLang Neural Network) is our original architecture featuring:

- **Gated Residual Blocks**: Each block learns a gate (sigmoid) that controls how much of the transformed signal vs. the skip connection to pass through. This allows the network to adaptively decide layer depth.
- **Multi-Scale Feature Mixer**: Input is processed through three parallel paths (narrow, medium, wide) with different activation functions, then concatenated — capturing patterns at multiple granularities.
- **Learnable Skip Connections**: Unlike standard ResNets with hardcoded identity skips, UniNN projects skip connections when dimensions change.

```
Input ──→ MultiScaleMixer ──→ [GatedResidualBlock × N] ──→ OutputHead ──→ Output
                                      │
                          gate ──→ σ ──┤
                                      │
                    transform ────────→ gate × transform
                         skip ────────→ (1-gate) × skip
                                      │
                                      └──→ output
```

## Quick Start

### Train a model

```unilang
from models.uniNN import UniNN
from core.loss import CrossEntropyLoss
from core.optimizers import Adam

// Create model
model = UniNN(
    inputDim=10,
    hiddenDim=64,
    outputDim=3,
    numBlocks=3,
    task="classification"
)

// Train
loss_fn = CrossEntropyLoss()
optimizer = Adam(model.parameters(), lr=0.001)

for epoch in range(100):
    model.zero_grad()
    predictions = model.forward(X_train)
    loss = loss_fn.compute(predictions, y_train)
    loss.backward()
    optimizer.step()

// Save
model.save("my_model.json")
```

### Load and predict

```unilang
model = UniNN.load("my_model.json")
model.eval_mode()
output = model.forward(input_tensor)
```

### Parallel ensemble (Java threads)

```unilang
from core.network import ParallelEnsemble

ensemble = ParallelEnsemble(
    models=[model1, model2, model3],
    strategy="weighted"
)
// Runs all 3 models on separate JVM threads
result = ensemble.predict(input)
```

### Time series prediction (LSTM)

```unilang
from core.layers import LSTM, Linear
from core.network import Sequential

model = Sequential("stock_predictor")
model.add(LSTM(inputDim=6, hiddenDim=64, numLayers=2))
model.add(Linear(64, 1))

// Input: [batch, 30 days, 6 features] → Output: [batch, 1 predicted price]
```

### Sensor anomaly detection (Conv1D)

```unilang
from core.layers import Conv1D, MaxPool1D, Linear, ReLU, Sigmoid

model = Sequential("anomaly_detector")
model.add(Conv1D(inChannels=4, outChannels=16, kernelSize=5, padding=2))
model.add(ReLU())
model.add(MaxPool1D(kernelSize=2))
model.add(Linear(160, 1))
model.add(Sigmoid())
```

## Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| Custom Tensor with autograd | Full control over computation graph and gradients |
| Cache-tiled matmul | 10-50x faster than naive loops on CPU |
| Java threading for parallelism | Real thread-level parallelism (no GIL) |
| JSON model serialization | Human-readable, language-agnostic, easy to inspect |
| Gated residuals over standard ResNet | Adaptive depth — network learns which layers matter |
| He initialization | Prevents vanishing gradients with ReLU |
| Cosine annealing scheduler | Smooth LR decay, better convergence |

## Components

### Tensor (`core/tensor.uniL`)
- Multi-dimensional array with flat storage
- Automatic differentiation (reverse-mode autograd)
- Operations: matmul, add, mul, sub, pow, sum, mean, transpose
- Activations: relu, sigmoid, tanh, leaky_relu, softmax
- Factory methods: zeros, ones, rand, randn, xavier_uniform, he_normal
- Cache-tiled matrix multiplication (32×32 tiles)

### Layers (`core/layers.uniL`)
- `Linear` — Fully connected with He initialization
- `ReLU`, `Sigmoid`, `Tanh`, `LeakyReLU`, `Softmax` — Activations
- `Dropout` — Inverted dropout regularization
- `BatchNorm` — Batch normalization with running statistics
- `Embedding` — Token embedding lookup table
- `LSTM` — Long Short-Term Memory for sequential/time series data (multi-layer, forget gate bias init)
- `Conv1D` — 1D convolution for local pattern detection in sequences
- `MaxPool1D` — 1D max pooling for downsampling sequences

### Loss Functions (`core/loss.uniL`)
- `MSELoss` — Mean squared error (regression)
- `CrossEntropyLoss` — Softmax + log-loss (classification)
- `BCELoss` — Binary cross-entropy
- `HuberLoss` — Robust regression

### Optimizers (`core/optimizers.uniL`)
- `SGD` — With momentum and weight decay
- `Adam` — AdamW variant with decoupled weight decay
- `RMSProp` — Adaptive learning rate
- `StepLRScheduler` — Step decay
- `CosineAnnealingScheduler` — Cosine annealing

## License

Apache License 2.0
