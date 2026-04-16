// ── Product Detail Page (PDP) ─────────────────────────────────

let pdpState = { product: null, selectedSize: '', selectedColor: '', qty: 1 };

async function renderPDP(productId) {
  const main = document.getElementById('mainContent');
  main.innerHTML = `
    <div class="pdp-container">
      <div style="margin-bottom:20px;">
        <a href="#" onclick="history.back(); return false;" style="font-size:.88rem;color:var(--grey-600);">← Back</a>
      </div>
      <div class="pdp-grid">
        <div class="pdp-images">
          <div class="pdp-main-image skeleton" id="pdpImageWrap" style="height:520px"></div>
        </div>
        <div class="pdp-info" id="pdpInfo">
          <div class="skeleton" style="height:24px;width:80px;margin-bottom:12px"></div>
          <div class="skeleton" style="height:40px;margin-bottom:16px"></div>
          <div class="skeleton" style="height:28px;width:160px;margin-bottom:32px"></div>
          <div class="skeleton" style="height:120px"></div>
        </div>
      </div>
      <div id="pdpReviews"></div>
      <div id="pdpRecs"></div>
    </div>`;

  try {
    const data = await API.getProduct(productId);
    const p = data.product;
    pdpState.product      = p;
    pdpState.selectedSize  = p.sizes?.[0] || '';
    pdpState.selectedColor = p.colors?.[0] || '';
    pdpState.qty           = 1;

    // Image
    const imgWrap = document.getElementById('pdpImageWrap');
    if (imgWrap) {
      imgWrap.className = 'pdp-main-image';
      imgWrap.innerHTML = `<img src="${p.imageUrl}" alt="${p.name}"
        onerror="this.src='https://images.unsplash.com/photo-1523381210434-271e8be1f52b?w=600'" />`;
    }

    // Info panel
    const info = document.getElementById('pdpInfo');
    if (info) info.innerHTML = buildPDPInfo(p);

    // Reviews
    const revEl = document.getElementById('pdpReviews');
    if (revEl) revEl.innerHTML = buildReviews(data.reviews || [], p.id);

    // Recommendations
    const recEl = document.getElementById('pdpRecs');
    if (recEl && data.recommendations?.length) {
      recEl.innerHTML = `
        <div class="section" style="padding-top:0">
          <div class="section-header">
            <div>
              <div class="section-title">You May Also Like</div>
              <div class="section-sub">Recommended by AI engine</div>
            </div>
          </div>
          <div class="product-grid">${data.recommendations.map(r => productCard(r)).join('')}</div>
        </div>`;
    }
  } catch (e) {
    console.error('PDP error:', e);
    main.innerHTML = `<div style="text-align:center;padding:80px;color:var(--error)">Product not found.</div>`;
  }
}

function buildPDPInfo(p) {
  return `
    <div class="pdp-brand">${p.brand}</div>
    <h1 class="pdp-name">${p.name}</h1>
    <div class="pdp-price-row">
      <span class="pdp-price">${fmt.price(p.price)}</span>
      ${p.originalPrice > p.price ? `<span class="pdp-original">${fmt.price(p.originalPrice)}</span>` : ''}
      ${p.discount > 0 ? `<span class="pdp-discount">-${p.discount}% OFF</span>` : ''}
    </div>
    <div style="margin-bottom:16px">${fmt.rating(p.rating, p.reviewCount)}</div>

    <div class="option-label">Size: <span id="selectedSize">${pdpState.selectedSize}</span></div>
    <div class="size-grid">
      ${(p.sizes || []).map(s => `
        <button class="size-btn ${s === pdpState.selectedSize ? 'selected' : ''}"
          onclick="pdpSelectSize('${s}')">${s}</button>`).join('')}
    </div>

    <div class="option-label">Color: <span id="selectedColor">${pdpState.selectedColor}</span></div>
    <div class="color-grid">
      ${(p.colors || []).map(c => `
        <button class="color-btn ${c === pdpState.selectedColor ? 'selected' : ''}"
          onclick="pdpSelectColor('${c}')">${c}</button>`).join('')}
    </div>

    <div class="qty-row">
      <div class="option-label" style="margin:0">Qty:</div>
      <div class="qty-control">
        <button onclick="pdpQty(-1)">−</button>
        <span id="pdpQtyVal">${pdpState.qty}</span>
        <button onclick="pdpQty(1)">+</button>
      </div>
    </div>

    <div class="pdp-actions">
      <button class="btn btn-dark" onclick="pdpAddToCart()">Add to Bag</button>
      <button class="btn btn-pink" onclick="pdpBuyNow()">Buy Now</button>
    </div>

    <div class="pdp-meta">
      <span>📦 In stock: ${p.stock} units</span>
      <span>🏷️ Category: ${p.category} › ${p.subcategory}</span>
      <span>🌐 ${p.gender === 'unisex' ? 'Unisex' : p.gender === 'women' ? "Women's" : "Men's"}</span>
    </div>

    <div style="margin-top:24px;padding:16px;background:var(--grey-100);border-radius:var(--radius-sm)">
      <div style="font-size:.85rem;font-weight:700;margin-bottom:8px">Product Description</div>
      <p style="font-size:.85rem;color:var(--grey-600);line-height:1.7">${p.description}</p>
    </div>`;
}

function pdpSelectSize(s) {
  pdpState.selectedSize = s;
  document.getElementById('selectedSize').textContent = s;
  document.querySelectorAll('.size-btn').forEach(b => b.classList.toggle('selected', b.textContent === s));
}

function pdpSelectColor(c) {
  pdpState.selectedColor = c;
  document.getElementById('selectedColor').textContent = c;
  document.querySelectorAll('.color-btn').forEach(b => b.classList.toggle('selected', b.textContent === c));
}

function pdpQty(delta) {
  pdpState.qty = Math.max(1, Math.min(10, pdpState.qty + delta));
  document.getElementById('pdpQtyVal').textContent = pdpState.qty;
}

async function pdpAddToCart() {
  const p = pdpState.product;
  if (!p) return;
  try {
    await API.addToCart({
      productId: p.id,
      size:      pdpState.selectedSize,
      color:     pdpState.selectedColor,
      quantity:  pdpState.qty,
    });
    app.updateCartCount();
    app.showToast('Added to bag! 🛍️', 'success');
  } catch (e) {
    app.showToast('Failed to add: ' + e.message, 'error');
  }
}

async function pdpBuyNow() {
  await pdpAddToCart();
  app.openCart();
}

function buildReviews(reviews, productId) {
  return `
    <div class="section" style="padding-top:0">
      <div class="section-header">
        <div><div class="section-title">Customer Reviews</div></div>
      </div>
      <div style="display:flex;flex-direction:column;gap:16px;margin-bottom:32px">
        ${reviews.length ? reviews.map(r => `
          <div style="padding:16px;background:var(--white);border-radius:var(--radius-sm);box-shadow:var(--shadow-sm)">
            <div style="display:flex;justify-content:space-between;margin-bottom:6px">
              <span style="font-weight:600;font-size:.9rem">${r.user_name || r.userName || 'Customer'}</span>
              <span style="color:var(--gold);font-size:.88rem">★ ${Number(r.rating).toFixed(1)}</span>
            </div>
            <p style="font-size:.85rem;color:var(--grey-600)">${r.comment || ''}</p>
          </div>`).join('') : '<p style="color:var(--grey-400)">No reviews yet. Be the first!</p>'}
      </div>

      <div style="background:var(--white);padding:24px;border-radius:var(--radius);box-shadow:var(--shadow-sm);max-width:500px">
        <h3 style="font-size:1rem;font-weight:700;margin-bottom:16px">Write a Review</h3>
        <div class="form-group">
          <label>Your Name</label>
          <input type="text" id="reviewName" placeholder="Enter your name" />
        </div>
        <div class="form-group">
          <label>Rating</label>
          <select id="reviewRating">
            <option value="5">★★★★★ (5)</option>
            <option value="4">★★★★ (4)</option>
            <option value="3">★★★ (3)</option>
            <option value="2">★★ (2)</option>
            <option value="1">★ (1)</option>
          </select>
        </div>
        <div class="form-group">
          <label>Comment</label>
          <input type="text" id="reviewComment" placeholder="Share your experience…" />
        </div>
        <button class="btn btn-dark" onclick="submitReview('${productId}')">Post Review</button>
      </div>
    </div>`;
}

async function submitReview(productId) {
  const name    = document.getElementById('reviewName')?.value.trim();
  const rating  = Number(document.getElementById('reviewRating')?.value);
  const comment = document.getElementById('reviewComment')?.value.trim();
  if (!name || !comment) { app.showToast('Please fill in all fields', 'error'); return; }
  try {
    await API.postReview(productId, { userName: name, rating, comment });
    app.showToast('Review posted! Thank you 🌟', 'success');
    renderPDP(productId);
  } catch (e) {
    app.showToast('Failed to post review', 'error');
  }
}

window.renderPDP       = renderPDP;
window.pdpSelectSize   = pdpSelectSize;
window.pdpSelectColor  = pdpSelectColor;
window.pdpQty          = pdpQty;
window.pdpAddToCart    = pdpAddToCart;
window.pdpBuyNow       = pdpBuyNow;
window.submitReview    = submitReview;
