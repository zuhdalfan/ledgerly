// Injected into every recorded page: a visible cursor + click ripple that
// follows the real mouse events Playwright dispatches. Playwright's video
// recording does not draw the OS cursor, so a walkthrough would otherwise be
// unfollowable. Development tooling only.
(() => {
  const build = () => {
    if (document.getElementById("__demo_cursor")) return true;
    const root = document.documentElement || document.body;
    if (!root) return false;

    const style = document.createElement("style");
    style.textContent = `
      #__demo_cursor { position: fixed; left: 0; top: 0; width: 22px; height: 32px;
        z-index: 2147483647; pointer-events: none;
        filter: drop-shadow(0 1px 2px rgba(0,0,0,.4)); }
      #__demo_ripple { position: fixed; left: 0; top: 0; width: 0; height: 0;
        border-radius: 50%; border: 2px solid rgba(47,45,42,.9);
        z-index: 2147483646; pointer-events: none; transform: translate(-50%,-50%); opacity: 0; }
    `;
    root.appendChild(style);

    const cursor = document.createElement("div");
    cursor.id = "__demo_cursor";
    cursor.innerHTML =
      '<svg viewBox="0 0 20 30" xmlns="http://www.w3.org/2000/svg"><path d="M1 1 L1 25 L7 19 L11 28 L15 26 L11 17 L19 17 Z" fill="#2F2D2A" stroke="#FBF8F2" stroke-width="2" stroke-linejoin="round"/></svg>';

    const ripple = document.createElement("div");
    ripple.id = "__demo_ripple";

    root.appendChild(cursor);
    root.appendChild(ripple);

    document.addEventListener(
      "mousemove",
      (e) => {
        cursor.style.left = `${e.clientX - 2}px`;
        cursor.style.top = `${e.clientY - 2}px`;
      },
      true
    );
    document.addEventListener(
      "mousedown",
      (e) => {
        ripple.style.left = `${e.clientX}px`;
        ripple.style.top = `${e.clientY}px`;
        ripple.style.transition = "none";
        ripple.style.width = "0px";
        ripple.style.height = "0px";
        ripple.style.opacity = "0.6";
        void ripple.offsetWidth;
        ripple.style.transition = "width .45s ease-out, height .45s ease-out, opacity .45s ease-out";
        ripple.style.width = "56px";
        ripple.style.height = "56px";
        ripple.style.opacity = "0";
      },
      true
    );
    window.__cursorReady = true;
    return true;
  };

  if (!build()) {
    document.addEventListener("DOMContentLoaded", build);
  }
})();
