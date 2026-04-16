// ── SHYNX App Controller ──────────────────────────────────────

const app = {
  currentView: 'home',

  async init() {
    this.bindEvents();
    await this.navigate('home');
    await this.updateCartCount();
  },

  bindEvents() {
    // Nav links
    document.querySelectorAll('[data-view]').forEach(el => {
      el.addEventListener('click', e => {
        e.preventDefault();
        const view = el.dataset.view;
        if (view === 'women')        this.navigatePLP({ gender: 'women', title: "Women's" });
        else if (view === 'men')     this.navigatePLP({ gender: 'men',   title: "Men's" });
        else if (view === 'trending')this.navigatePLP({ trending: true,  title: 'Trending' });
        else if (view === 'new-arrivals') this.navigatePLP({ new: true,  title: 'New Arrivals' });
        else if (view === 'sale')    this.navigatePLP({ sort: 'price_asc', title: 'Sale' });
        else                         this.navigate(view);
      });
    });

    // Logo
    document.getElementById('logoBtn')?.addEventListener('click', e => {
      e.preventDefault();
      this.navigate('home');
    });

    // Cart
    document.getElementById('cartBtn')?.addEventListener('click', () => this.openCart());
    document.getElementById('closeCart')?.addEventListener('click', () => this.closeCart());
    document.getElementById('cartOverlay')?.addEventListener('click', () => this.closeCart());

    // Auth
    document.getElementById('profileBtn')?.addEventListener('click', () => this.openAuth());
    document.getElementById('closeAuth')?.addEventListener('click', () => this.closeAuth());
    document.getElementById('authOverlay')?.addEventListener('click', () => this.closeAuth());

    // Mobile nav toggle
    document.getElementById('navToggle')?.addEventListener('click', () => {
      document.getElementById('mainNav')?.classList.toggle('open');
    });

    // Search
    const searchInput = document.getElementById('searchInput');
    const dropdown    = document.getElementById('searchDropdown');

    searchInput?.addEventListener('input', debounce(async (e) => {
      const q = e.target.value.trim();
      if (q.length < 2) { dropdown.classList.remove('active'); return; }
      try {
        const data = await API.search(q);
        const items = (data.products || []).slice(0, 6);
        if (!items.length) { dropdown.classList.remove('active'); return; }
        dropdown.innerHTML = items.map(p => `
          <div class="search-result-item" onclick="app.showPDP('${p.id}'); document.getElementById('searchInput').value=''; document.getElementById('searchDropdown').classList.remove('active')">
            <img src="${p.imageUrl}" alt="${p.name}" onerror="this.src='https://images.unsplash.com/photo-1523381210434-271e8be1f52b?w=100'" />
            <div class="search-result-info">
              <div class="name">${p.name}</div>
              <div class="brand">${p.brand}</div>
              <div class="price">${fmt.price(p.price)}</div>
            </div>
          </div>`).join('');
        dropdown.classList.add('active');
      } catch {}
    }, 300));

    document.getElementById('searchBtn')?.addEventListener('click', () => {
      const q = document.getElementById('searchInput')?.value.trim();
      if (q) { this.navigatePLP({ search: q, title: `Search: "${q}"` }); dropdown.classList.remove('active'); }
    });

    searchInput?.addEventListener('keydown', e => {
      if (e.key === 'Enter') {
        const q = e.target.value.trim();
        if (q) { this.navigatePLP({ search: q, title: `Search: "${q}"` }); dropdown.classList.remove('active'); }
      }
    });

    document.addEventListener('click', e => {
      if (!e.target.closest('#searchBarContainer') && !e.target.closest('#searchDropdown')) {
        dropdown?.classList.remove('active');
      }
    });

    // Wishlist btn
    document.getElementById('wishlistBtn')?.addEventListener('click', () => {
      this.showToast('Sign in to view wishlist', '');
    });
  },

  async navigate(view) {
    this.currentView = view;
    this.updateActiveNav(view);
    document.getElementById('mainNav')?.classList.remove('open');
    window.scrollTo({ top: 0, behavior: 'smooth' });

    switch (view) {
      case 'home':     await renderHome();     break;
      case 'checkout': await renderCheckout(); break;
      case 'orders':   await renderOrders();   break;
      default:         await renderHome();
    }
  },

  async navigatePLP(opts = {}) {
    this.currentView = 'plp';
    this.updateActiveNav('');
    document.getElementById('mainNav')?.classList.remove('open');
    window.scrollTo({ top: 0, behavior: 'smooth' });
    await renderPLP(opts);
  },

  async showPDP(productId) {
    this.currentView = 'pdp';
    this.updateActiveNav('');
    window.scrollTo({ top: 0, behavior: 'smooth' });
    await renderPDP(productId);
  },

  updateActiveNav(view) {
    document.querySelectorAll('.nav-link').forEach(el => {
      el.classList.toggle('active', el.dataset.view === view);
    });
  },

  async openCart() {
    document.getElementById('cartSidebar')?.classList.add('open');
    document.getElementById('cartOverlay')?.classList.add('active');
    document.body.style.overflow = 'hidden';
    await loadCart();
  },

  closeCart() {
    document.getElementById('cartSidebar')?.classList.remove('open');
    document.getElementById('cartOverlay')?.classList.remove('active');
    document.body.style.overflow = '';
  },

  async updateCartCount() {
    try {
      const data = await API.getCart();
      updateCartBadge(data.itemCount || 0);
    } catch {}
  },

  openAuth(mode = 'login') {
    const overlay = document.getElementById('authOverlay');
    const modal   = document.getElementById('authModal');
    const content = document.getElementById('authContent');
    if (!content) return;
    content.innerHTML = mode === 'login' ? buildLoginForm() : buildRegisterForm();
    overlay?.classList.add('active');
    modal?.classList.add('active');
  },

  closeAuth() {
    document.getElementById('authOverlay')?.classList.remove('active');
    document.getElementById('authModal')?.classList.remove('active');
  },

  showToast(msg, type = '') {
    const toast = document.getElementById('toast');
    if (!toast) return;
    toast.textContent = msg;
    toast.className = 'toast show ' + type;
    clearTimeout(this._toastTimer);
    this._toastTimer = setTimeout(() => toast.classList.remove('show'), 3000);
  },

  async quickAdd(productId) {
    try {
      const data = await API.getProduct(productId);
      const p = data.product;
      await API.addToCart({
        productId: p.id,
        size:  p.sizes?.[0] || '',
        color: p.colors?.[0] || '',
        quantity: 1,
      });
      await this.updateCartCount();
      this.showToast(`${p.name} added to bag 🛍️`, 'success');
    } catch (e) {
      this.showToast('Could not add item', 'error');
    }
  },

  toggleWishlist(productId) {
    this.showToast('Added to wishlist ♥', 'success');
  },
};

// ── Auth Forms ───────────────────────────────────────────────

function buildLoginForm() {
  return `
    <div class="auth-title">Welcome Back</div>
    <div class="auth-sub">Sign in to your SHYNX account</div>
    <div class="form-group"><label>Email</label><input type="email" id="loginEmail" placeholder="you@email.com" /></div>
    <div class="form-group"><label>Password</label><input type="password" id="loginPass" placeholder="••••••••" /></div>
    <button class="btn btn-dark" style="width:100%;justify-content:center;padding:14px" onclick="submitLogin()">Sign In</button>
    <div class="auth-toggle">Don't have an account? <a onclick="app.openAuth('register')">Register</a></div>`;
}

function buildRegisterForm() {
  return `
    <div class="auth-title">Create Account</div>
    <div class="auth-sub">Join SHYNX and unlock exclusive deals</div>
    <div class="form-group"><label>Name</label><input type="text" id="regName" placeholder="Your name" /></div>
    <div class="form-group"><label>Email</label><input type="email" id="regEmail" placeholder="you@email.com" /></div>
    <div class="form-group"><label>Phone</label><input type="tel" id="regPhone" placeholder="+91 98765 43210" /></div>
    <button class="btn btn-pink" style="width:100%;justify-content:center;padding:14px" onclick="submitRegister()">Create Account</button>
    <div class="auth-toggle">Already have an account? <a onclick="app.openAuth('login')">Sign In</a></div>`;
}

async function submitLogin() {
  const email = document.getElementById('loginEmail')?.value.trim();
  const pass  = document.getElementById('loginPass')?.value;
  if (!email) { app.showToast('Enter your email', 'error'); return; }
  try {
    const data = await API.login({ email, password: pass });
    localStorage.setItem('shynx_token',   data.token);
    localStorage.setItem('shynx_user_id', data.userId);
    localStorage.setItem('shynx_name',    data.name);
    app.closeAuth();
    app.showToast(`Welcome back, ${data.name}! 👋`, 'success');
  } catch (e) {
    app.showToast(e.message, 'error');
  }
}

async function submitRegister() {
  const name  = document.getElementById('regName')?.value.trim();
  const email = document.getElementById('regEmail')?.value.trim();
  const phone = document.getElementById('regPhone')?.value.trim();
  if (!name || !email) { app.showToast('Fill in required fields', 'error'); return; }
  try {
    const data = await API.register({ name, email, phone });
    localStorage.setItem('shynx_token',   data.token);
    localStorage.setItem('shynx_user_id', data.userId);
    localStorage.setItem('shynx_name',    data.name);
    app.closeAuth();
    app.showToast(`Welcome to SHYNX, ${data.name}! 🎉`, 'success');
  } catch (e) {
    app.showToast(e.message, 'error');
  }
}

// ── Utility ──────────────────────────────────────────────────

function debounce(fn, delay) {
  let t;
  return (...args) => { clearTimeout(t); t = setTimeout(() => fn(...args), delay); };
}

window.submitLogin    = submitLogin;
window.submitRegister = submitRegister;

// ── Boot ─────────────────────────────────────────────────────

document.addEventListener('DOMContentLoaded', () => app.init());
