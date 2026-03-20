# 6. Best Practices

> **Goal**: Practical tips that save you hours of debugging and produce reliable models.

---

## Data Preparation

### Always normalize your input features

```
DON'T: Feed raw values with wildly different scales
  Feature 1 (salary):     $50,000 - $200,000
  Feature 2 (age):        18 - 65
  Feature 3 (rating):     1.0 - 5.0
  → The model thinks salary is 10,000× more important than rating

DO: Normalize to zero mean, unit variance
  Feature 1 (salary):     -1.5 to +1.5
  Feature 2 (age):        -1.8 to +1.8
  Feature 3 (rating):     -1.2 to +1.2
  → All features start on equal footing
```

### Shuffle your data

If data is ordered (e.g., all "buy" customers first, then all "leave" customers), the model learns the order instead of the patterns. Always shuffle before training.

### Split your data before anything else

```
Full dataset (1000 samples)
├── Training set (800 samples) → Model learns from this
├── Validation set (100 samples) → You monitor progress with this
└── Test set (100 samples) → Final evaluation (touch ONCE at the very end)

NEVER let the model see test data during training.
NEVER tune hyperparameters based on test accuracy.
```

---

## Model Design

### Start small, then grow

```
DON'T: Start with the biggest model you can fit in memory.

DO:
  1. Start:   hiddenDim=16, numBlocks=1     → Does it learn anything?
  2. Grow:    hiddenDim=32, numBlocks=2     → Does accuracy improve?
  3. Grow:    hiddenDim=64, numBlocks=3     → Still improving?
  4. Stop:    When test accuracy stops improving or starts dropping
```

### Recommended starting configurations

| Dataset size | Features | Hidden dim | Blocks | Dropout |
|-------------|----------|-----------|--------|---------|
| < 500 | < 10 | 16-32 | 1-2 | 0.0-0.1 |
| 500 - 5,000 | 10-50 | 32-64 | 2-3 | 0.1-0.2 |
| 5,000 - 50,000 | 50-200 | 64-128 | 3-4 | 0.1-0.3 |
| > 50,000 | 200+ | 128-256 | 3-5 | 0.2-0.4 |

---

## Training

### Learning rate is the single most impactful hyperparameter

```
Try these in order:
  1. lr=0.001  (default for Adam — start here)
  2. lr=0.01   (if loss barely moves)
  3. lr=0.0001 (if loss is unstable or diverging)
  4. lr=0.005  (if 0.001 works but slowly)
```

### Use learning rate scheduling

Start high (fast learning), end low (fine-tuning):

```unilang
scheduler = CosineAnnealingScheduler(optimizer, totalSteps=epochs, minLR=1e-5)
```

### Monitor training vs validation loss

```
Healthy training:
  Train loss: 0.8 → 0.3 → 0.1 → 0.05
  Val loss:   0.9 → 0.4 → 0.15 → 0.12    (gap is small)

Overfitting:
  Train loss: 0.8 → 0.3 → 0.01 → 0.001   (keeps dropping)
  Val loss:   0.9 → 0.4 → 0.5  → 0.8     (starts RISING) ← STOP HERE
```

When validation loss starts increasing while training loss keeps decreasing, stop training. You've entered overfitting territory.

### Save checkpoints

Save the model at the best validation loss, not at the end of training:

```unilang
best_val_loss = float('inf')

for epoch in range(epochs):
    // ... train ...

    val_loss = evaluate(model, X_val, y_val)
    if val_loss < best_val_loss:
        best_val_loss = val_loss
        model.save("best_model.json")     // Save the best version
        print(f"  New best model saved (val_loss={val_loss:.4f})")
```

---

## Debugging

### If loss is NaN

1. **Reduce learning rate** — most common cause
2. **Check for infinity in data** — one bad value corrupts everything
3. **Check for division by zero** — especially in normalization
4. **Gradient clipping** — cap gradients to prevent explosion:
   ```unilang
   for param in model.parameters():
       for i in range(param.size):
           param.grad[i] = max(-1.0, min(1.0, param.grad[i]))
   ```

### If loss doesn't decrease

1. **Verify forward pass** — print shapes at each layer
2. **Verify backward pass** — check that gradients are non-zero:
   ```unilang
   for param in model.parameters():
       grad_sum = sum(abs(param.grad[i]) for i in range(param.size))
       print(f"{param.name}: grad_sum={grad_sum:.6f}")
   // If grad_sum is 0 for all parameters, backward() isn't working
   ```
3. **Check data labels** — are they correct? Is one-hot encoding right?
4. **Try a simpler model** — can `Linear(5, 3) + Softmax` learn anything?

### If model only predicts one class

The model has collapsed — it predicts the most common class for everything.

1. **Check class balance** — if 90% of data is class A, the model gets 90% accuracy by always predicting A
2. **Use class weights** in loss function or oversample minority classes
3. **Increase learning rate** — the model might be stuck in a local minimum

---

## Code Quality

### Name your tensors and layers

```unilang
// DON'T:
model = UniNN(5, 32, 3)

// DO:
model = UniNN(
    inputDim=5,          // What goes in
    hiddenDim=32,        // Processing width
    outputDim=3,         // What comes out
    numBlocks=2,         // Depth
    task="classification"
)
```

### Log everything during training

```unilang
print(f"Epoch {epoch:4d}/{epochs} | "
      f"Train Loss: {train_loss:.4f} | "
      f"Val Loss: {val_loss:.4f} | "
      f"Val Acc: {val_acc:.4f} | "
      f"LR: {optimizer.learningRate:.6f} | "
      f"Time: {elapsed:.1f}s")
```

### Version your models

```
models/trained/
├── customer_classifier_v1.json     ← first attempt
├── customer_classifier_v2.json     ← after tuning
├── customer_classifier_v3.json     ← after adding features
└── customer_classifier_v3_best.json ← best validation score
```

---

## Quick Reference Checklist

Before training, verify:

- [ ] Data is normalized (zero mean, unit variance)
- [ ] Data is shuffled
- [ ] Train/validation/test split is done
- [ ] No data leakage (test data not used for training)
- [ ] Model architecture matches problem (classification vs regression)
- [ ] Loss function matches task (CrossEntropy for classification, MSE for regression)
- [ ] Learning rate is reasonable (start with 0.001)

During training, monitor:

- [ ] Loss is decreasing
- [ ] Validation loss is not diverging from training loss
- [ ] Gradients are non-zero and not exploding
- [ ] Saving best checkpoint based on validation loss

After training, verify:

- [ ] Test accuracy is close to validation accuracy
- [ ] Model predicts all classes (not just the majority)
- [ ] Predictions make intuitive sense on manual examples

---

**Next**: [7. API Reference →](./07_API_REFERENCE.md)
