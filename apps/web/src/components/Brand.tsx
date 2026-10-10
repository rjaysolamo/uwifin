export function Brand({ light = false }: { light?: boolean }) {
  return (
    <span className={`brand ${light ? 'brand-light' : ''}`}>
      <svg width="34" height="34" viewBox="0 0 34 34" fill="none" aria-hidden="true">
        <rect width="34" height="34" rx="10" fill="currentColor" />
        <path
          d="M9 13v6a8 8 0 0 0 16 0v-6M13 11v8a4 4 0 0 0 8 0v-8"
          stroke="var(--brand-mark, white)"
          strokeWidth="2.5"
          strokeLinecap="round"
        />
        <path
          d="m22 11 3-3 3 3"
          stroke="var(--brand-mark, white)"
          strokeWidth="2.3"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
      </svg>
      <span>
        uwifin<span className="brand-dot">.</span>
      </span>
    </span>
  );
}
