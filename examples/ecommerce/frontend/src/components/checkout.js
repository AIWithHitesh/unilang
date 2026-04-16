// ── Checkout Page ─────────────────────────────────────────────

let checkoutData = { cart: null, paymentMethod: 'upi' };

async function renderCheckout() {
  const main = document.getElementById('mainContent');

  let cartData;
  try {
    cartData = await API.getCart();
    checkoutData.cart = cartData;
  } catch (e) {
    main.innerHTML = `<div style="text-align:center;padding:80px;color:var(--error)">Failed to load cart.</div>`;
    return;
  }

  if (!cartData.items?.length) {
    main.innerHTML = `
      <div class="order-confirm">
        <div class="confirm-icon">🛍️</div>
        <h1>Your bag is empty</h1>
        <p>Add some items before checking out.</p>
        <button class="btn btn-dark" style="margin-top:24px" onclick="app.navigate('home')">Continue Shopping</button>
      </div>`;
    return;
  }

  const subtotal  = cartData.total || 0;
  const shipping  = subtotal >= 999 ? 0 : 99;
  const grandTotal = subtotal + shipping;

  main.innerHTML = `
    <div class="checkout-container">
      <div class="checkout-form">
        <h2>Checkout</h2>

        <div class="form-section">
          <h3>📦 Delivery Address</h3>
          <div class="form-row">
            <div class="form-group">
              <label>First Name</label>
              <input type="text" id="chkFirstName" placeholder="Priya" />
            </div>
            <div class="form-group">
              <label>Last Name</label>
              <input type="text" id="chkLastName" placeholder="Sharma" />
            </div>
          </div>
          <div class="form-group">
            <label>Email</label>
            <input type="email" id="chkEmail" placeholder="priya@email.com" />
          </div>
          <div class="form-group">
            <label>Phone</label>
            <input type="tel" id="chkPhone" placeholder="+91 98765 43210" />
          </div>
          <div class="form-group">
            <label>Address Line 1</label>
            <input type="text" id="chkAddr1" placeholder="123, Koramangala" />
          </div>
          <div class="form-group">
            <label>Address Line 2 (optional)</label>
            <input type="text" id="chkAddr2" placeholder="Near Starbucks" />
          </div>
          <div class="form-row">
            <div class="form-group">
              <label>City</label>
              <input type="text" id="chkCity" placeholder="Bengaluru" />
            </div>
            <div class="form-group">
              <label>PIN Code</label>
              <input type="text" id="chkPin" placeholder="560034" />
            </div>
          </div>
          <div class="form-group">
            <label>State</label>
            <select id="chkState">
              ${['Karnataka','Maharashtra','Delhi','Tamil Nadu','Telangana','Gujarat','Rajasthan','West Bengal','Uttar Pradesh','Pune']
                .map(s => `<option>${s}</option>`).join('')}
            </select>
          </div>
        </div>

        <div class="form-section">
          <h3>💳 Payment Method</h3>
          <div class="payment-methods">
            ${[
              ['upi',  '📱', 'UPI / Google Pay / PhonePe'],
              ['card', '💳', 'Credit / Debit Card'],
              ['cod',  '💵', 'Cash on Delivery'],
              ['bnpl', '🔖', 'Buy Now Pay Later'],
            ].map(([val, icon, label]) => `
              <div class="payment-option ${val === 'upi' ? 'selected' : ''}"
                   onclick="selectPayment('${val}')" data-method="${val}">
                <span class="payment-icon">${icon}</span>
                <span class="payment-label">${label}</span>
              </div>`).join('')}
          </div>

          <div id="cardDetails" style="display:none;margin-top:16px">
            <div class="form-group">
              <label>Card Number</label>
              <input type="text" placeholder="•••• •••• •••• ••••" maxlength="19" />
            </div>
            <div class="form-row">
              <div class="form-group">
                <label>Expiry</label>
                <input type="text" placeholder="MM/YY" maxlength="5" />
              </div>
              <div class="form-group">
                <label>CVV</label>
                <input type="password" placeholder="•••" maxlength="3" />
              </div>
            </div>
          </div>
        </div>

        <button class="btn btn-pink" style="width:100%;justify-content:center;padding:18px;font-size:1rem;margin-top:8px"
          onclick="placeOrder()">
          Place Order — ${fmt.price(grandTotal)}
        </button>

        <p style="text-align:center;font-size:.78rem;color:var(--grey-400);margin-top:12px">
          🔒 Secured by 256-bit SSL encryption
        </p>
      </div>

      <div class="order-summary">
        <h2>Order Summary</h2>
        <div class="order-items">
          ${(cartData.items || []).map(it => `
            <div class="order-item">
              <img src="${it.imageUrl || ''}" alt="${it.name}"
                onerror="this.src='https://images.unsplash.com/photo-1523381210434-271e8be1f52b?w=100'" />
              <div class="order-item-info">
                <div class="name">${it.name}</div>
                <div class="meta">${[it.size, it.color].filter(Boolean).join(' · ')} × ${it.quantity}</div>
              </div>
              <div class="order-item-price">${fmt.price(it.price * it.quantity)}</div>
            </div>`).join('')}
        </div>
        <div class="order-divider"></div>
        <div class="cart-row"><span>Subtotal</span><span>${fmt.price(subtotal)}</span></div>
        <div class="cart-row">
          <span>Shipping</span>
          <span>${shipping === 0 ? '<span style="color:var(--success);font-weight:700">FREE</span>' : fmt.price(shipping)}</span>
        </div>
        <div class="order-divider"></div>
        <div class="cart-row total"><span>Total</span><span>${fmt.price(grandTotal)}</span></div>

        <div style="margin-top:20px;padding:14px;background:rgba(34,197,94,.08);border-radius:var(--radius-sm);border:1px solid rgba(34,197,94,.2)">
          <div style="font-size:.82rem;font-weight:700;color:var(--success);margin-bottom:4px">📦 Free Delivery</div>
          <div style="font-size:.78rem;color:var(--grey-600)">Estimated delivery: 3-5 business days</div>
        </div>
      </div>
    </div>
    ${renderFooter()}`;
}

function selectPayment(method) {
  checkoutData.paymentMethod = method;
  document.querySelectorAll('.payment-option').forEach(el => {
    el.classList.toggle('selected', el.dataset.method === method);
  });
  const cardEl = document.getElementById('cardDetails');
  if (cardEl) cardEl.style.display = method === 'card' ? 'block' : 'none';
}

async function placeOrder() {
  const firstName = document.getElementById('chkFirstName')?.value.trim();
  const lastName  = document.getElementById('chkLastName')?.value.trim();
  const email     = document.getElementById('chkEmail')?.value.trim();
  const phone     = document.getElementById('chkPhone')?.value.trim();
  const addr1     = document.getElementById('chkAddr1')?.value.trim();
  const city      = document.getElementById('chkCity')?.value.trim();
  const pin       = document.getElementById('chkPin')?.value.trim();
  const state     = document.getElementById('chkState')?.value;

  if (!firstName || !email || !addr1 || !city || !pin) {
    app.showToast('Please fill in all required fields', 'error');
    return;
  }

  const address = { name: `${firstName} ${lastName}`, email, phone, line1: addr1, city, pin, state };

  const btn = document.querySelector('.checkout-container .btn-pink');
  if (btn) { btn.disabled = true; btn.textContent = 'Placing Order…'; }

  try {
    const order = await API.checkout({ address, paymentMethod: checkoutData.paymentMethod });
    app.updateCartCount();
    renderOrderConfirmation(order);
  } catch (e) {
    app.showToast('Order failed: ' + e.message, 'error');
    if (btn) { btn.disabled = false; btn.textContent = 'Place Order'; }
  }
}

function renderOrderConfirmation(order) {
  const main = document.getElementById('mainContent');
  main.innerHTML = `
    <div class="order-confirm">
      <div class="confirm-icon">✅</div>
      <h1>Order Confirmed!</h1>
      <p>Thank you for shopping with SHYNX 🎉</p>
      <p class="order-id">${order.id}</p>
      <p style="margin-top:8px">Estimated delivery: <strong>${order.estimatedDelivery}</strong></p>
      <p>A confirmation will be sent to <strong>${order.address?.email || 'your email'}</strong></p>

      <div style="margin:32px auto;padding:20px;background:var(--white);border-radius:var(--radius);max-width:400px;box-shadow:var(--shadow-sm)">
        <div style="font-size:.85rem;font-weight:700;margin-bottom:12px;text-align:left">Order Summary</div>
        ${(order.items || []).map(it => `
          <div style="display:flex;justify-content:space-between;font-size:.85rem;padding:6px 0;border-bottom:1px solid var(--grey-200)">
            <span>${it.name} × ${it.quantity}</span>
            <span style="font-weight:600">${fmt.price(it.price * it.quantity)}</span>
          </div>`).join('')}
        <div style="display:flex;justify-content:space-between;font-size:1rem;font-weight:700;padding:12px 0 0">
          <span>Total Paid</span>
          <span>${fmt.price(order.total)}</span>
        </div>
      </div>

      <div style="display:flex;gap:12px;justify-content:center;flex-wrap:wrap">
        <button class="btn btn-dark" onclick="app.navigate('home')">Continue Shopping</button>
        <button class="btn btn-outline" style="border-color:var(--black);color:var(--black)"
          onclick="app.navigate('orders')">View My Orders</button>
      </div>
    </div>
    ${renderFooter()}`;
}

async function renderOrders() {
  const main = document.getElementById('mainContent');
  main.innerHTML = `
    <div style="max-width:800px;margin:40px auto;padding:0 24px">
      <h1 style="font-family:var(--font-display);font-size:2rem;margin-bottom:32px">My Orders</h1>
      <div id="ordersList"><div class="skeleton" style="height:120px;margin-bottom:16px"></div></div>
    </div>`;

  try {
    const orders = await API.getOrders();
    const el = document.getElementById('ordersList');
    if (!el) return;
    if (!orders.length) {
      el.innerHTML = `<p style="color:var(--grey-400);text-align:center;padding:40px">No orders yet.</p>`;
      return;
    }
    el.innerHTML = orders.map(o => `
      <div style="background:var(--white);padding:20px;border-radius:var(--radius);margin-bottom:16px;box-shadow:var(--shadow-sm)">
        <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:8px">
          <span style="font-weight:700">${o.id}</span>
          <span style="padding:4px 12px;border-radius:20px;font-size:.78rem;font-weight:700;
            background:${o.status === 'CONFIRMED' ? 'rgba(34,197,94,.1)' : 'var(--grey-100)'};
            color:${o.status === 'CONFIRMED' ? 'var(--success)' : 'var(--grey-600)'}">
            ${o.status}
          </span>
        </div>
        <div style="font-size:.85rem;color:var(--grey-600)">${o.itemCount} item(s) · ${fmt.price(o.total)}</div>
        <div style="font-size:.78rem;color:var(--grey-400);margin-top:4px">${o.createdAt}</div>
      </div>`).join('');
  } catch (e) {
    console.error(e);
  }
}

window.renderCheckout          = renderCheckout;
window.renderOrders            = renderOrders;
window.selectPayment           = selectPayment;
window.placeOrder              = placeOrder;
window.renderOrderConfirmation = renderOrderConfirmation;
