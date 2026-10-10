/// Bounded rendering for the list pages.
///
/// Measured, not assumed: with 10,000 invoices the backend returns the whole
/// set in ~43ms (`cargo test -- --ignored --nocapture shows_how_the_list_scales`),
/// so paging in SQL would buy nothing. What actually gets slow is the DOM —
/// 10,000 rows is roughly 70,000 cells — so the list renders a generous prefix
/// and says plainly what it is holding back.
///
/// This is a deliberate trade: no windowing library, no paging state, no
/// scroll-position bugs, and a number the user can see rather than a silently
/// incomplete table.
export const RENDER_LIMIT = 250;

export function capped<T>(rows: T[]): { shown: T[]; hidden: number } {
  if (rows.length <= RENDER_LIMIT) return { shown: rows, hidden: 0 };
  return { shown: rows.slice(0, RENDER_LIMIT), hidden: rows.length - RENDER_LIMIT };
}
