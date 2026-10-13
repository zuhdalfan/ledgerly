// Marketing overlay, injected only for marketing recordings (never for the
// README walkthrough). Adds a brand chip, a caption card and an end card on top
// of the real app, so one recording already carries the message.
//
// Everything is pointer-events:none — the overlay never blocks the app.
// Development tooling only.
(() => {
  const LOGO = `
    <svg viewBox="0 0 512 512" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
      <rect x="0" y="0" width="512" height="512" rx="112" fill="#2F2D2A" />
      <g transform="translate(256 260) scale(4.6) translate(-48 -40)">
        <path d="M24 22A10 10 0 0 1 34 12H62A10 10 0 0 1 72 22V62L66 68 60 62 54 68 48 62 42 68 36 62 30 68 24 62Z" fill="#FBF8F2"/>
        <path d="M35 31H61" stroke="#2F2D2A" stroke-width="5" stroke-linecap="round"/>
        <path d="M35 44H61" stroke="#2F2D2A" stroke-width="5" stroke-linecap="round"/>
      </g>
    </svg>`;

  const css = `
    #__mk { position: fixed; inset: 0; z-index: 2147483600; pointer-events: none;
      font-family: Inter, ui-sans-serif, system-ui, sans-serif; -webkit-font-smoothing: antialiased; }
    #__mk * { box-sizing: border-box; }

    /* Sizes scale with the frame, so 16:9, 1:1 and 9:16 all read the same. */
    #__mk { --pad: clamp(18px, 4vmin, 56px); --title: clamp(22px, 4.4vmin, 58px);
            --sub: clamp(15px, 2.4vmin, 30px); --chip: clamp(15px, 2.1vmin, 26px); }

    #__mk_brand { position: absolute; top: var(--pad); left: var(--pad);
      display: flex; align-items: center; gap: 0.5em; padding: 0.45em 0.9em 0.45em 0.5em;
      background: #FBF8F2; border-radius: 999px; box-shadow: 0 6px 22px rgba(47,45,42,.18);
      font-size: var(--chip); font-weight: 700; color: #2F2D2A; opacity: 0; transition: opacity .5s ease; }
    #__mk_brand.on { opacity: 1; }
    #__mk_brand svg { width: 1.7em; height: 1.7em; display: block; }

    #__mk_caption { position: absolute; left: var(--pad); right: var(--pad); bottom: var(--pad);
      display: flex; flex-direction: column; align-items: center; text-align: center; gap: .35em;
      opacity: 0; transform: translateY(14px); transition: opacity .4s ease, transform .4s ease; }
    #__mk_caption.on { opacity: 1; transform: translateY(0); }
    #__mk_caption .box { max-width: min(92%, 46em); padding: .55em 1.1em .7em;
      background: #2F2D2A; border-radius: 1.1em; box-shadow: 0 14px 40px rgba(47,45,42,.28); }
    #__mk_caption .t { display: block; font-size: var(--title); font-weight: 700;
      letter-spacing: -.02em; line-height: 1.15; color: #FBF8F2; }
    #__mk_caption .s { display: block; margin-top: .35em; font-size: var(--sub);
      line-height: 1.35; color: #D9D4C8; }

    #__mk_end { position: absolute; inset: 0; display: flex; flex-direction: column;
      align-items: center; justify-content: center; gap: .5em; text-align: center;
      padding: var(--pad); background: #F5F1E8; opacity: 0; transition: opacity .5s ease; }
    #__mk_end.on { opacity: 1; }
    #__mk_end svg { width: clamp(64px, 13vmin, 132px); height: auto; }
    #__mk_end .t { margin-top: .5em; font-size: clamp(26px, 5.6vmin, 74px); font-weight: 700;
      letter-spacing: -.03em; color: #2F2D2A; }
    #__mk_end .s { font-size: clamp(15px, 2.6vmin, 34px); color: #6B6659; }
    #__mk_end .c { margin-top: .6em; font-size: clamp(14px, 2.2vmin, 28px); font-weight: 700;
      color: #2F2D2A; border: 2px solid #2F2D2A; border-radius: 999px; padding: .35em 1em; }
  `;

  const build = () => {
    if (document.getElementById("__mk")) return true;
    const root = document.documentElement || document.body;
    if (!root) return false;

    const style = document.createElement("style");
    style.textContent = css;
    root.appendChild(style);

    const wrap = document.createElement("div");
    wrap.id = "__mk";
    wrap.innerHTML = `
      <div id="__mk_brand">${LOGO}<span>Ledgerly</span></div>
      <div id="__mk_caption"><div class="box"><span class="t"></span><span class="s"></span></div></div>
      <div id="__mk_end">
        ${LOGO}
        <div class="t"></div>
        <div class="s"></div>
        <div class="c"></div>
      </div>`;
    root.appendChild(wrap);

    const brand = wrap.querySelector("#__mk_brand");
    const cap = wrap.querySelector("#__mk_caption");
    const capT = cap.querySelector(".t");
    const capS = cap.querySelector(".s");
    const end = wrap.querySelector("#__mk_end");

    window.__marketing = {
      brand(show = true) {
        brand.classList.toggle("on", show);
      },
      caption(title, sub = "") {
        capT.textContent = title;
        capS.textContent = sub;
        capS.style.display = sub ? "block" : "none";
        cap.classList.add("on");
      },
      clearCaption() {
        cap.classList.remove("on");
      },
      endCard(title, sub = "", cta = "") {
        cap.classList.remove("on");
        end.querySelector(".t").textContent = title;
        end.querySelector(".s").textContent = sub;
        end.querySelector(".c").textContent = cta;
        end.querySelector(".c").style.display = cta ? "inline-block" : "none";
        requestAnimationFrame(() => end.classList.add("on"));
      },
    };
    window.__marketingReady = true;
    return true;
  };

  if (!build()) document.addEventListener("DOMContentLoaded", build);
})();
