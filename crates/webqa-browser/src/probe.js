(el, arg) => {
  const r = el.getBoundingClientRect();
  const left = Math.max(r.left, 0);
  const right = Math.min(r.right, window.innerWidth);
  const top = Math.max(r.top, 0);
  const bottom = Math.min(r.bottom, window.innerHeight);
  const at = right > left && bottom > top
    ? document.elementFromPoint((left + right) / 2, (top + bottom) / 2)
    : null;
  const hit = at !== null && (at === el || el.contains(at));
  const href = typeof el.href === 'string' && el.href !== '' ? el.href : null;
  const own = el.getAttribute('formaction');
  const form = el.form instanceof HTMLFormElement ? el.form.action : null;
  const formAction = own !== null ? new URL(own, document.baseURI).href : form;
  const listed = (selectors) => selectors.some((s) => el.closest(s) !== null);
  return { hit, href, formAction, forbidden: listed(arg.forbidden), allowed: listed(arg.allowed) };
}
