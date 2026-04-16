// ── SHYNX API Client ──────────────────────────────────────────

const API_BASE = 'http://localhost:8080/api';

// Generate or retrieve session ID
function getSessionId() {
  let sid = localStorage.getItem('shynx_session');
  if (!sid) {
    sid = 'sess-' + Date.now() + '-' + Math.random().toString(36).slice(2, 9);
    localStorage.setItem('shynx_session', sid);
  }
  return sid;
}

async function apiFetch(path, options = {}) {
  const headers = {
    'Content-Type': 'application/json',
    'X-Session-Id': getSessionId(),
    ...(options.headers || {}),
  };
  const token = localStorage.getItem('shynx_token');
  if (token) headers['Authorization'] = 'Bearer ' + token;

  const res = await fetch(API_BASE + path, { ...options, headers });
  const data = await res.json().catch(() => ({}));
  if (!res.ok) throw new Error(data.error || 'Request failed');
  return data;
}

const API = {
  // Products
  getProducts:     (params = {}) => apiFetch('/products?' + new URLSearchParams(params)),
  getProduct:      (id)          => apiFetch(`/products/${id}`),
  getCategories:   ()            => apiFetch('/categories'),
  getTrending:     (limit = 12)  => apiFetch(`/trending?limit=${limit}`),
  getNewArrivals:  (limit = 12)  => apiFetch(`/new-arrivals?limit=${limit}`),
  search:          (q)           => apiFetch(`/search?q=${encodeURIComponent(q)}`),

  // Recommendations
  getPlpRecs:  (category = '', gender = '', limit = 12) =>
    apiFetch(`/recommendations?category=${category}&gender=${gender}&limit=${limit}`),
  getPdpRecs:  (productId, limit = 8) =>
    apiFetch(`/products/${productId}/recommendations?limit=${limit}`),

  // Cart
  getCart:     ()                => apiFetch('/cart'),
  addToCart:   (item)            => apiFetch('/cart', { method: 'POST', body: JSON.stringify(item) }),
  updateCart:  (item)            => apiFetch('/cart', { method: 'PUT',  body: JSON.stringify(item) }),
  clearCart:   ()                => apiFetch('/cart', { method: 'DELETE' }),

  // Checkout & Orders
  checkout:    (data)            => apiFetch('/checkout', { method: 'POST', body: JSON.stringify(data) }),
  getOrders:   ()                => apiFetch('/orders'),
  getOrder:    (id)              => apiFetch(`/orders/${id}`),

  // Auth
  register:    (data)            => apiFetch('/auth/register', { method: 'POST', body: JSON.stringify(data) }),
  login:       (data)            => apiFetch('/auth/login',    { method: 'POST', body: JSON.stringify(data) }),

  // Reviews
  postReview:  (productId, data) => apiFetch(`/products/${productId}/reviews`, { method: 'POST', body: JSON.stringify(data) }),

  // Wishlist
  addWishlist:    (data)     => apiFetch('/wishlist',   { method: 'POST',   body: JSON.stringify(data) }),
  removeWishlist: (data)     => apiFetch('/wishlist',   { method: 'DELETE', body: JSON.stringify(data) }),
  getWishlist:    (userId)   => apiFetch(`/wishlist/${userId}`),
};

window.API = API;
window.getSessionId = getSessionId;

// ── Formatting helpers ────────────────────────────────────────
window.fmt = {
  price: (n) => '₹' + Number(n).toLocaleString('en-IN'),
  rating: (r, count) => `<span class="stars">★</span> ${Number(r).toFixed(1)} <span>(${count})</span>`,
  badge: (p) => {
    let b = '';
    if (p.isTrending) b += '<span class="badge badge-trending">Trending</span>';
    if (p.isNew)      b += '<span class="badge badge-new">New</span>';
    if (p.discount > 20) b += `<span class="badge badge-sale">-${p.discount}%</span>`;
    return b;
  },
};

// ── Product Card template ─────────────────────────────────────
window.productCard = (p, opts = {}) => `
  <div class="product-card" data-id="${p.id}" onclick="app.showPDP('${p.id}')">
    <div class="product-image-wrap">
      <img src="${p.imageUrl}" alt="${p.name}" loading="lazy" onerror="this.src='https://images.unsplash.com/photo-1523381210434-271e8be1f52b?w=400'" />
      <div class="product-badges">${fmt.badge(p)}</div>
      <button class="wishlist-toggle" onclick="event.stopPropagation(); app.toggleWishlist('${p.id}')" aria-label="Wishlist">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/></svg>
      </button>
      <div class="quick-add" onclick="event.stopPropagation(); app.quickAdd('${p.id}')">Quick Add +</div>
    </div>
    <div class="product-info">
      <div class="product-brand">${p.brand}</div>
      <div class="product-name">${p.name}</div>
      <div class="product-price">
        <span class="price-current">${fmt.price(p.price)}</span>
        ${p.originalPrice > p.price ? `<span class="price-original">${fmt.price(p.originalPrice)}</span>` : ''}
        ${p.discount > 0 ? `<span class="price-discount">-${p.discount}%</span>` : ''}
      </div>
      <div class="product-rating">${fmt.rating(p.rating, p.reviewCount)}</div>
    </div>
  </div>`;
