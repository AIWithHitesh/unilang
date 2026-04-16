// ── Cart Sidebar ──────────────────────────────────────────────

async function loadCart() {
  try {
    const data = await API.getCart();
    renderCartSidebar(data);
    updateCartBadge(data.itemCount || 0);
  } catch (e) {
    console.error('Cart load error:', e);
  }
}

function renderCartSidebar(data) {
  const items   = data.items || [];
  const total   = data.total || 0;
  const count   = data.itemCount || 0;
  const label   = document.getElementById('cartItemLabel');
  const itemsEl = document.getElementById('cartItems');
  const footerEl= document.getElementById('cartFooter');

  if (label) label.textContent = count > 0 ? `(${count})` : '';

  if (!itemsEl) return;

  if (items.length === 0) {
    itemsEl.innerHTML = `
      <div class="cart-empty">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M6 2 3 6v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V6l-3-4z"/>
          <line x1="3" y1="6" x2="21" y2="6"/>
        </svg>
        <h3>Your bag is empty</h3>
        <p>Add some fabulous pieces!</p>
        <button class="btn btn-dark" style="margin-top:16px" onclick="app.closeCart(); app.navigate('women')">Start Shopping</button>
      </div>`;
    if (footerEl) footerEl.innerHTML = '';
    return;
  }

  itemsEl.innerHTML = items.map((it, idx) => `
    <div class="cart-item">
      <img src="${it.imageUrl || 'https://images.unsplash.com/photo-1523381210434-271e8be1f52b?w=200'}"
           alt="${it.name}"
           onerror="this.src='https://images.unsplash.com/photo-1523381210434-271e8be1f52b?w=200'" />
      <div class="cart-item-info">
        <div class="cart-item-name">${it.name}</div>
        <div class="cart-item-meta">${[it.size, it.color].filter(Boolean).join(' · ')}</div>
        <div class="cart-item-price">${fmt.price(it.price)}</div>
        <div class="cart-item-qty">
          <div class="qty-control qty-sm">
            <button onclick="cartUpdateQty(${idx}, -1)">−</button>
            <span>${it.quantity}</span>
            <button onclick="cartUpdateQty(${idx}, 1)">+</button>
          </div>
          <button class="cart-item-remove" onclick="cartRemove(${idx})">Remove</button>
        </div>
      </div>
    </div>`).join('');

  const shipping = total >= 999 ? 0 : 99;
  const grand = total + shipping;

  if (footerEl) {
    footerEl.innerHTML = `
      <div class="cart-summary">
        <div class="cart-row"><span>Subtotal</span><span>${fmt.price(total)}</span></div>
        <div class="cart-row"><span>Shipping</span><span>${shipping === 0 ? '<span style="color:var(--success)">FREE</span>' : fmt.price(shipping)}</span></div>
        ${shipping > 0 ? `<div style="font-size:.75rem;color:var(--grey-400);margin-top:-4px">Add ${fmt.price(999 - total)} more for free shipping</div>` : ''}
        <div class="cart-row total"><span>Total</span><span>${fmt.price(grand)}</span></div>
      </div>
      <button class="btn btn-dark" style="width:100%;justify-content:center;padding:16px;font-size:1rem"
        onclick="app.closeCart(); app.navigate('checkout')">
        Proceed to Checkout →
      </button>
      <button class="btn" style="width:100%;justify-content:center;padding:10px;margin-top:8px;color:var(--grey-600);font-size:.85rem"
        onclick="app.closeCart()">Continue Shopping</button>`;
  }
}

async function cartUpdateQty(idx, delta) {
  try {
    const data = await API.getCart();
    const item = data.items[idx];
    if (!item) return;
    const newQty = Math.max(0, item.quantity + delta);
    await API.updateCart({ productId: item.productId, size: item.size, color: item.color, quantity: newQty });
    await loadCart();
  } catch (e) {
    app.showToast('Update failed', 'error');
  }
}

async function cartRemove(idx) {
  try {
    const data = await API.getCart();
    const item = data.items[idx];
    if (!item) return;
    await API.updateCart({ productId: item.productId, size: item.size, color: item.color, quantity: 0 });
    await loadCart();
  } catch (e) {
    app.showToast('Remove failed', 'error');
  }
}

function updateCartBadge(count) {
  const el = document.getElementById('cartCount');
  if (el) el.textContent = count;
}

window.loadCart       = loadCart;
window.renderCartSidebar = renderCartSidebar;
window.cartUpdateQty  = cartUpdateQty;
window.cartRemove     = cartRemove;
window.updateCartBadge = updateCartBadge;
