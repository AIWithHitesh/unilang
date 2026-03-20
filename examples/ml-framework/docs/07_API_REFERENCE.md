# 7. API Reference

> Complete reference for every class and method in the UniLang ML Framework.

---

## core/tensor.uniL — Tensor

The fundamental data structure. All numbers flow through Tensors.

### Construction

| Method | Signature | Description |
|--------|-----------|-------------|
| `Tensor()` | `Tensor(int[] shape, bool requiresGrad=false)` | Create zero-filled tensor |
| `zeros()` | `Tensor.zeros(int[] shape, bool requiresGrad=false)` | All zeros |
| `ones()` | `Tensor.ones(int[] shape, bool requiresGrad=false)` | All ones |
| `fill()` | `Tensor.fill(int[] shape, double value)` | Fill with constant |
| `rand()` | `Tensor.rand(int[] shape)` | Uniform random [0, 1) |
| `randn()` | `Tensor.randn(int[] shape)` | Normal distribution N(0, 1) |
| `xavier_uniform()` | `Tensor.xavier_uniform(int fan_in, int fan_out)` | Xavier/Glorot init |
| `he_normal()` | `Tensor.he_normal(int fan_in, int fan_out)` | He/Kaiming init |
| `from_list()` | `Tensor.from_list(list data, int[] shape=None)` | Create from Python list |

### Math Operations (all support autograd)

| Method | Signature | Description |
|--------|-----------|-------------|
| `matmul()` | `matmul(Tensor other) → Tensor` | Matrix multiply `[M,K] @ [K,N] → [M,N]` |
| `add()` | `add(Tensor other) → Tensor` | Element-wise add (with broadcasting) |
| `sub()` | `sub(Tensor other) → Tensor` | Element-wise subtract |
| `mul()` | `mul(Tensor other) → Tensor` | Element-wise multiply |
| `scale()` | `scale(double scalar) → Tensor` | Multiply all elements by scalar |
| `neg()` | `neg() → Tensor` | Negate all elements |
| `pow()` | `pow(double exponent) → Tensor` | Element-wise power |
| `sum()` | `sum() → Tensor` | Sum all elements → scalar tensor |
| `mean()` | `mean() → Tensor` | Mean of all elements → scalar tensor |
| `transpose()` | `transpose() → Tensor` | 2D matrix transpose |

### Activation Functions (all support autograd)

| Method | Signature | Output Range | Use Case |
|--------|-----------|-------------|----------|
| `relu()` | `relu() → Tensor` | [0, ∞) | Default for hidden layers |
| `sigmoid()` | `sigmoid() → Tensor` | (0, 1) | Binary classification output |
| `tanh()` | `tanh() → Tensor` | (-1, 1) | Hidden layers (alternative to ReLU) |
| `leaky_relu()` | `leaky_relu(double alpha=0.01) → Tensor` | (-∞, ∞) | When ReLU "dies" (all zeros) |
| `softmax()` | `softmax() → Tensor` | (0, 1), sums to 1 | Multi-class classification output |

### Autograd

| Method | Description |
|--------|-------------|
| `backward()` | Compute gradients for all tensors in computation graph |
| `zero_grad()` | Reset `.grad` array to zeros |

### Utility

| Method | Description |
|--------|-------------|
| `clone() → Tensor` | Deep copy |
| `reshape(int[] shape) → Tensor` | Change shape (must have same total size) |
| `to_list() → list` | Convert data to Python list |
| `getShape() → list` | Get dimensions |
| `getSize() → int` | Get total element count |

---

## core/layers.uniL — Neural Network Layers

All layers implement the `Layer` interface:

```unilang
public interface Layer {
    Tensor forward(Tensor input);
    list parameters();
    String name();
}
```

### Linear (Fully Connected / Dense)

```unilang
layer = Linear(inFeatures=10, outFeatures=64, name="hidden1")
output = layer.forward(input)    // [batch, 10] → [batch, 64]
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `inFeatures` | int | Number of input features |
| `outFeatures` | int | Number of output features (neurons) |
| `name` | String | Layer name for debugging |

Initialization: He normal (good for ReLU networks).

### ReLU / Sigmoid / Tanh / LeakyReLU / Softmax

```unilang
relu = ReLU()
output = relu.forward(input)    // Same shape, negatives zeroed
```

No parameters. These are pure functions wrapped as layers.

`LeakyReLU(alpha=0.01)` — `alpha` controls the slope for negative inputs.

### Dropout

```unilang
dropout = Dropout(rate=0.5)
dropout.set_training(true)       // Enable during training
dropout.set_training(false)      // Disable during evaluation
output = dropout.forward(input)  // Randomly zeros 50% of values
```

Uses inverted dropout: scales surviving values by `1/(1-rate)` so output magnitude stays consistent.

### BatchNorm

```unilang
bn = BatchNorm(numFeatures=64, epsilon=1e-5, momentum=0.1)
output = bn.forward(input)      // [batch, 64] → [batch, 64] (normalized)
```

Maintains running mean/variance for inference. Has learnable `gamma` (scale) and `beta` (shift) parameters.

### Embedding

```unilang
embed = Embedding(vocabSize=10000, embeddingDim=128)
output = embed.forward(token_ids)   // [batch×seq_len] → [batch×seq_len, 128]
```

Lookup table mapping integer IDs to dense vectors.

---

## core/network.uniL — Network Containers

### Sequential

```unilang
model = Sequential("my_model")
model.add(Linear(10, 64))
model.add(ReLU())
model.add(Linear(64, 3))
model.add(Softmax())

output = model.forward(input)       // Passes through all layers in order
params = model.parameters()         // All trainable parameters
model.summary()                     // Print architecture table
model.save("model.json")           // Save weights
model.load("model.json")           // Load weights
model.train_mode()                  // Enable dropout/batchnorm training behavior
model.eval_mode()                   // Disable dropout, use running stats for batchnorm
```

### ParallelEnsemble

```unilang
ensemble = ParallelEnsemble(
    models=[model1, model2, model3],
    strategy="average"              // "average" or "weighted"
)
ensemble.weights = [0.5, 0.3, 0.2] // For weighted strategy
result = ensemble.predict(input)     // Runs all models in parallel via Java threads
```

---

## core/loss.uniL — Loss Functions

All loss functions implement:

```unilang
public interface LossFunction {
    Tensor compute(Tensor predictions, Tensor targets);
    String name();
}
```

| Loss | Constructor | Input → Output |
|------|------------|----------------|
| `MSELoss()` | No params | `([batch, N], [batch, N]) → [1]` |
| `CrossEntropyLoss()` | No params | `([batch, classes], [batch, classes]) → [1]` |
| `BCELoss()` | No params | `([batch, 1], [batch, 1]) → [1]` |
| `HuberLoss(delta=1.0)` | `delta`: threshold | `([batch, N], [batch, N]) → [1]` |

**Note**: `CrossEntropyLoss` expects raw logits (pre-softmax) OR softmax probabilities. It includes numerical stability (log-sum-exp trick) internally.

---

## core/optimizers.uniL — Optimizers

All optimizers implement:

```unilang
public interface Optimizer {
    void step();       // Update parameters using gradients
    void zero_grad();  // Reset all parameter gradients
    String name();
}
```

### SGD

```unilang
optimizer = SGD(params, lr=0.01, momentum=0.9, weightDecay=1e-4)
```

| Parameter | Default | Description |
|-----------|---------|-------------|
| `lr` | 0.01 | Learning rate |
| `momentum` | 0.0 | Momentum factor (0.9 is common) |
| `weightDecay` | 0.0 | L2 regularization strength |

### Adam

```unilang
optimizer = Adam(params, lr=0.001, beta1=0.9, beta2=0.999, epsilon=1e-8, weightDecay=0.0)
```

| Parameter | Default | Description |
|-----------|---------|-------------|
| `lr` | 0.001 | Learning rate |
| `beta1` | 0.9 | First moment decay (gradient memory) |
| `beta2` | 0.999 | Second moment decay (gradient magnitude memory) |
| `epsilon` | 1e-8 | Numerical stability |
| `weightDecay` | 0.0 | AdamW-style decoupled weight decay |

### RMSProp

```unilang
optimizer = RMSProp(params, lr=0.01, alpha=0.99, epsilon=1e-8)
```

### Learning Rate Schedulers

```unilang
// Step decay: multiply lr by gamma every stepSize epochs
scheduler = StepLRScheduler(optimizer, stepSize=10, gamma=0.1)

// Cosine annealing: smooth decay from base lr to min lr
scheduler = CosineAnnealingScheduler(optimizer, totalSteps=100, minLR=1e-6)

// Use in training loop:
for epoch in range(epochs):
    // ... train ...
    scheduler.step()
```

---

## models/uniNN.uniL — UniNN Model

```unilang
model = UniNN(
    inputDim=10,            // Number of input features
    hiddenDim=64,           // Hidden layer width
    outputDim=3,            // Number of output classes/values
    numBlocks=3,            // Number of gated residual blocks
    dropoutRate=0.1,        // Dropout probability
    task="classification"   // "classification" or "regression"
)
```

| Method | Description |
|--------|-------------|
| `forward(Tensor input) → Tensor` | Run inference |
| `parameters() → list` | All trainable parameters |
| `num_parameters() → int` | Total parameter count |
| `zero_grad()` | Reset all gradients |
| `train_mode()` | Enable dropout/batchnorm training |
| `eval_mode()` | Disable dropout, use running stats |
| `summary()` | Print architecture table |
| `save(String path)` | Save model to JSON |
| `UniNN.load(String path) → UniNN` | Load model from JSON (static method) |

---

## core/trainer.uniL — Training Utilities

### Trainer

```unilang
trainer = Trainer(model, loss_fn, optimizer)

results = trainer.fit(
    X_train, y_train,
    epochs=100,
    batchSize=32,
    X_val=X_val,        // Optional validation data
    y_val=y_val,
    printEvery=10,      // Print every N epochs
    shuffle=true
)

// results = {"history": [...], "totalTime": 12.3, "finalLoss": 0.05, "finalValAccuracy": 0.94}
```

### Helper Functions

```unilang
from core.trainer import create_batches, shuffle_data

batches = create_batches(X, y, batchSize=32)
X_shuffled, y_shuffled = shuffle_data(X, y)
```

---

**Next**: [8. Use Cases →](./08_USE_CASES.md)
