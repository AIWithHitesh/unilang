// ── Product Listing Page (PLP) ────────────────────────────────

let plpState = { page: 1, pageSize: 24, total: 0, totalPages: 0, params: {} };

async function renderPLP(opts = {}) {
  plpState.params = opts;
  plpState.page   = opts.page || 1;

  const main = document.getElementById('mainContent');
  const title = opts.category || opts.gender || opts.title || 'All Products';
  const showRecs = !!opts.category;

  main.innerHTML = `
    <div class="plp-container">
      <div class="plp-header">
        <div>
          <h1 class="plp-title">${title}</h1>
          <p class="plp-count" id="plpCount">Loading…</p>
        </div>
      </div>

      <!-- Recommendations strip (PLP) -->
      ${showRecs ? `
      <div style="margin-bottom:32px;">
        <div style="font-size:.8rem;font-weight:700;color:var(--grey-400);text-transform:uppercase;letter-spacing:.08em;margin-bottom:12px;">
          🤖 Recommended for You in ${opts.category}
        </div>
        <div class="product-grid" id="plpRecGrid" style="grid-template-columns:repeat(auto-fill,minmax(180px,1fr))">
          ${[...Array(4)].map(() => '<div class="skeleton skeleton-card" style="height:280px"></div>').join('')}
        </div>
      </div>
      <hr style="border:none;border-top:1px solid var(--grey-200);margin-bottom:32px;" />
      ` : ''}

      <div class="filter-bar">
        <span class="filter-label">Filter:</span>
        <select class="filter-select" id="fCategory" onchange="plpFilter()">
          <option value="">All Categories</option>
          ${['Dresses','Tops','Bottoms','Footwear','Bags','Accessories','Outerwear','Activewear','Swimwear','Ethnic','Kids']
            .map(c => `<option value="${c}" ${opts.category === c ? 'selected' : ''}>${c}</option>`).join('')}
        </select>
        <select class="filter-select" id="fGender" onchange="plpFilter()">
          <option value="">All Genders</option>
          <option value="women" ${opts.gender === 'women' ? 'selected' : ''}>Women</option>
          <option value="men"   ${opts.gender === 'men'   ? 'selected' : ''}>Men</option>
          <option value="unisex">Unisex</option>
        </select>
        <select class="filter-select" id="fSort" onchange="plpFilter()">
          <option value="">Recommended</option>
          <option value="rating">Top Rated</option>
          <option value="price_asc">Price: Low to High</option>
          <option value="price_desc">Price: High to Low</option>
          <option value="newest">New Arrivals</option>
        </select>
        <label style="display:flex;align-items:center;gap:6px;font-size:.85rem;cursor:pointer;">
          <input type="checkbox" id="fTrending" onchange="plpFilter()" ${opts.trending ? 'checked' : ''}> Trending
        </label>
        <label style="display:flex;align-items:center;gap:6px;font-size:.85rem;cursor:pointer;">
          <input type="checkbox" id="fNew" onchange="plpFilter()" ${opts.new ? 'checked' : ''}> New In
        </label>
      </div>

      <div class="product-grid large" id="plpGrid">
        ${[...Array(8)].map(() => '<div class="skeleton skeleton-card"></div>').join('')}
      </div>

      <div class="pagination" id="plpPagination"></div>
    </div>
    ${renderFooter()}
  `;

  await loadPLPProducts();

  if (showRecs) {
    try {
      const recs = await API.getPlpRecs(opts.category || '', opts.gender || '', 6);
      const g = document.getElementById('plpRecGrid');
      if (g) g.innerHTML = (recs.recommendations || []).map(p => productCard(p)).join('');
    } catch (e) {}
  }
}

async function loadPLPProducts() {
  const params = {
    page:     plpState.page,
    pageSize: plpState.pageSize,
    ...(plpState.params.category ? { category: plpState.params.category } : {}),
    ...(plpState.params.gender   ? { gender:   plpState.params.gender   } : {}),
    ...(plpState.params.search   ? { search:   plpState.params.search   } : {}),
    ...(plpState.params.sort     ? { sort:     plpState.params.sort     } : {}),
    ...(plpState.params.trending ? { trending: 'true' } : {}),
    ...(plpState.params.new      ? { new:      'true' } : {}),
  };

  try {
    const data = await API.getProducts(params);
    plpState.total      = data.total;
    plpState.totalPages = data.totalPages;

    const count = document.getElementById('plpCount');
    if (count) count.textContent = `${data.total} products`;

    const grid = document.getElementById('plpGrid');
    if (grid) {
      grid.innerHTML = data.products.length
        ? data.products.map(p => productCard(p)).join('')
        : '<p style="grid-column:1/-1;text-align:center;color:var(--grey-400);padding:60px">No products found.</p>';
    }

    renderPLPPagination(data.page, data.totalPages);
  } catch (e) {
    console.error('PLP load error:', e);
    const grid = document.getElementById('plpGrid');
    if (grid) grid.innerHTML = '<p style="grid-column:1/-1;text-align:center;color:var(--error)">Failed to load products. Is the server running?</p>';
  }
}

function plpFilter() {
  const cat      = document.getElementById('fCategory')?.value || '';
  const gender   = document.getElementById('fGender')?.value   || '';
  const sort     = document.getElementById('fSort')?.value     || '';
  const trending = document.getElementById('fTrending')?.checked;
  const isNew    = document.getElementById('fNew')?.checked;
  plpState.params = { ...plpState.params, category: cat, gender, sort, trending, new: isNew };
  plpState.page = 1;
  loadPLPProducts();
}

function renderPLPPagination(current, total) {
  const el = document.getElementById('plpPagination');
  if (!el || total <= 1) return;
  let html = '';
  html += `<button class="page-btn" onclick="plpGoto(${current - 1})" ${current === 1 ? 'disabled style="opacity:.4"' : ''}>‹</button>`;
  const start = Math.max(1, current - 2);
  const end   = Math.min(total, current + 2);
  for (let i = start; i <= end; i++) {
    html += `<button class="page-btn ${i === current ? 'active' : ''}" onclick="plpGoto(${i})">${i}</button>`;
  }
  html += `<button class="page-btn" onclick="plpGoto(${current + 1})" ${current === total ? 'disabled style="opacity:.4"' : ''}>›</button>`;
  el.innerHTML = html;
}

function plpGoto(page) {
  if (page < 1 || page > plpState.totalPages) return;
  plpState.page = page;
  loadPLPProducts();
  window.scrollTo({ top: 0, behavior: 'smooth' });
}

window.renderPLP  = renderPLP;
window.plpFilter  = plpFilter;
window.plpGoto    = plpGoto;
