// ── Home Page ─────────────────────────────────────────────────

async function renderHome() {
  const main = document.getElementById('mainContent');
  main.innerHTML = `
    <!-- Hero -->
    <section class="hero">
      <div class="hero-inner">
        <div class="hero-text">
          <div class="hero-badge">✦ New Collection 2026</div>
          <h1>Dress to <span class="accent">Impress</span> Every Day</h1>
          <p>Discover 100+ curated fashion pieces from the world's top brands. Your style, your rules.</p>
          <div class="hero-actions">
            <button class="btn btn-primary" onclick="app.navigate('women')">Shop Women</button>
            <button class="btn btn-outline" onclick="app.navigate('men')">Shop Men</button>
          </div>
          <div class="hero-stats">
            <div class="hero-stat"><div class="number">100+</div><div class="label">Products</div></div>
            <div class="hero-stat"><div class="number">50+</div><div class="label">Brands</div></div>
            <div class="hero-stat"><div class="number">4.7★</div><div class="label">Avg Rating</div></div>
          </div>
        </div>
        <div class="hero-images">
          <div class="hero-img-card">
            <img src="https://images.unsplash.com/photo-1515372039744-b8f02a3ae446?w=600" alt="Fashion" />
          </div>
          <div class="hero-img-card">
            <img src="https://images.unsplash.com/photo-1541099649105-f69ad21f3246?w=600" alt="Fashion" />
          </div>
        </div>
      </div>
    </section>

    <!-- Categories -->
    <div class="section">
      <div class="section-header">
        <div><div class="section-title">Shop by Category</div></div>
      </div>
      <div class="category-grid" id="catGrid">
        ${['Dresses','Tops','Bottoms','Footwear','Bags','Accessories','Outerwear','Activewear','Swimwear','Ethnic','Kids']
          .map(c => `<button class="category-pill" onclick="app.navigatePLP({category:'${c}'})">${c}</button>`).join('')}
      </div>
    </div>

    <!-- Trending -->
    <div class="section">
      <div class="section-header">
        <div>
          <div class="section-title">🔥 Trending Now</div>
          <div class="section-sub">What everyone's obsessed with</div>
        </div>
        <a href="#" class="view-all" onclick="event.preventDefault(); app.navigate('trending')">View All →</a>
      </div>
      <div class="product-grid" id="trendingGrid">
        ${[...Array(4)].map(() => '<div class="skeleton skeleton-card"></div>').join('')}
      </div>
    </div>

    <!-- PLP Recs Banner -->
    <div class="rec-strip">
      <div class="section">
        <div class="section-header">
          <div>
            <div class="section-title">🤖 AI Picks for You</div>
            <div class="section-sub">Powered by UniLang Recommendation Engine</div>
          </div>
        </div>
        <div class="product-grid" id="recGrid">
          ${[...Array(4)].map(() => '<div class="skeleton skeleton-card"></div>').join('')}
        </div>
      </div>
    </div>

    <!-- New Arrivals -->
    <div class="section">
      <div class="section-header">
        <div>
          <div class="section-title">✨ New Arrivals</div>
          <div class="section-sub">Just dropped — be the first to wear it</div>
        </div>
        <a href="#" class="view-all" onclick="event.preventDefault(); app.navigate('new-arrivals')">View All →</a>
      </div>
      <div class="product-grid" id="newGrid">
        ${[...Array(4)].map(() => '<div class="skeleton skeleton-card"></div>').join('')}
      </div>
    </div>

    <!-- Banner -->
    <div class="banner-strip">
      <h2>Up to 60% Off Sale</h2>
      <p>Limited time. Limited stock. Unlimited style.</p>
      <button class="btn btn-primary" onclick="app.navigatePLP({sort:'price_asc'})">Shop the Sale</button>
    </div>

    <!-- Features -->
    <div class="features-bar">
      ${[
        ['🚚','Free Shipping','On orders above ₹999'],
        ['↩️','Easy Returns','30-day hassle-free returns'],
        ['🔒','Secure Payment','100% safe & encrypted'],
        ['⭐','Top Brands','500+ verified brands'],
      ].map(([icon, title, text]) => `
        <div class="feature-item">
          <div class="feature-icon">${icon}</div>
          <div><div class="feature-title">${title}</div><div class="feature-text">${text}</div></div>
        </div>`).join('')}
    </div>

    ${renderFooter()}
  `;

  // Load data in parallel
  try {
    const [trending, newArr, recs] = await Promise.all([
      API.getTrending(8),
      API.getNewArrivals(8),
      API.getPlpRecs('', '', 8),
    ]);

    document.getElementById('trendingGrid').innerHTML =
      trending.map(p => productCard(p)).join('');
    document.getElementById('newGrid').innerHTML =
      newArr.map(p => productCard(p)).join('');
    document.getElementById('recGrid').innerHTML =
      (recs.recommendations || []).map(p => productCard(p)).join('');
  } catch (e) {
    console.error('Home load error:', e);
  }
}

function renderFooter() {
  return `
    <footer class="footer">
      <div class="footer-grid">
        <div class="footer-brand">
          <span class="logo">SHYNX</span>
          <p>Fashion forward. Sustainably made. Built entirely with UniLang — showcasing full-stack development without Python or Node.js.</p>
        </div>
        <div class="footer-col">
          <h4>Shop</h4>
          <ul>
            <li><a href="#" onclick="app.navigate('women')">Women</a></li>
            <li><a href="#" onclick="app.navigate('men')">Men</a></li>
            <li><a href="#" onclick="app.navigate('trending')">Trending</a></li>
            <li><a href="#" onclick="app.navigate('new-arrivals')">New In</a></li>
          </ul>
        </div>
        <div class="footer-col">
          <h4>Help</h4>
          <ul>
            <li><a href="#">Size Guide</a></li>
            <li><a href="#">Returns</a></li>
            <li><a href="#">Track Order</a></li>
            <li><a href="#">Contact Us</a></li>
          </ul>
        </div>
        <div class="footer-col">
          <h4>About</h4>
          <ul>
            <li><a href="#">About SHYNX</a></li>
            <li><a href="#">Sustainability</a></li>
            <li><a href="#">Careers</a></li>
            <li><a href="#">Press</a></li>
          </ul>
        </div>
      </div>
      <div class="footer-bottom">
        <span>© 2026 SHYNX. Built with UniLang.</span>
        <span>Privacy · Terms · Cookies</span>
      </div>
    </footer>`;
}

window.renderHome = renderHome;
window.renderFooter = renderFooter;
