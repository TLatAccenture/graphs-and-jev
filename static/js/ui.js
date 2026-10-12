"use strict";
/* Shared page behaviour that is not data access (js/api.js owns that). */
(function () {
  /* Scrolls just enough to show something the reader has just asked for, such as a demo result
     or an opened layer. Nothing moves when it is already fully on screen.
     Options:
     - sticky: a sticky companion, such as the landing page's graph, that must not be pushed up
       while it is sticky; ignored when the layout has made it static.
     - opener: the control that opened the element, kept on screen above it.
     The top edge normally stops at the page's scroll padding below the sticky header; it may use
     that margin when that is what shows the whole element. Anything taller than the screen is
     aligned to the scroll padding instead, and when the element and a sticky companion cannot
     both fit, the element wins. The page's scroll-behavior decides smooth or instant, and
     guide.css makes it instant under reduced motion. Keyboard focus does not move. It measures on
     the next frame, after the page has finished updating the status line and anything else
     around the element. */
  function reveal(element, options) {
    requestAnimationFrame(() => place(element, options || {}));
  }

  function place(element, { sticky, opener }) {
    if (!element || element.hidden) return;
    const pad = parseFloat(getComputedStyle(document.documentElement).scrollPaddingTop) || 0;
    const bar = document.querySelector(".site-header");
    const edge = bar ? Math.max(0, bar.getBoundingClientRect().bottom) : 0;
    const screen = window.innerHeight;
    const box = element.getBoundingClientRect();
    // Measure where the element settles, not where its entrance animation has it right now.
    const shift = new DOMMatrixReadOnly(getComputedStyle(element).transform).m42;
    let top = box.top - shift;
    const bottom = box.bottom - shift;
    if (opener) top = Math.min(top, opener.getBoundingClientRect().top);
    let limit = Infinity; // scrolling further than this would push the sticky companion up
    if (sticky && getComputedStyle(sticky).position === "sticky") {
      const room = (parseFloat(getComputedStyle(sticky).top) || 0) + sticky.offsetHeight;
      limit = sticky.parentElement.getBoundingClientRect().bottom - room;
    }
    const least = bottom - screen; // scroll at least this far to show the bottom edge
    let most = Math.min(top - pad, limit); // and at most this far to keep the top edge clear
    if (least > most) most = Math.min(top - edge, limit);
    let by;
    if (least <= most) by = Math.max(least, Math.min(0, most));
    else by = bottom - top > screen - edge ? top - pad : least;
    if (Math.abs(by) >= 1) window.scrollBy({ top: by });
  }

  window.siteUi = { reveal };
})();
