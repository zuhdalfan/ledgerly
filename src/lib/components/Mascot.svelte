<script lang="ts">
  // Tally: a paper slip torn off the pad. Paper is the artifact this product is
  // about, so the mascot is the artifact.
  //
  // Colours come from CSS classes rather than presentation attributes, because
  // var() in an SVG attribute is not reliably supported.
  //
  // Decorative by default. Pass `label` only when it carries meaning alone.
  export type Mood =
    | "idle"
    | "pleased"
    | "worried"
    | "celebrating"
    | "waving"
    | "thinking"
    | "sleeping";

  let {
    expression = "idle",
    size = 72,
    label = "",
  }: { expression?: Mood; size?: number; label?: string } = $props();

  const animated = $derived(
    expression === "pleased" ||
      expression === "celebrating" ||
      expression === "waving" ||
      expression === "thinking"
  );
</script>

<svg
  class="mascot"
  class:bob={animated}
  width={size}
  height={size}
  viewBox="0 0 96 96"
  role={label ? "img" : "presentation"}
  aria-label={label || undefined}
  aria-hidden={label ? undefined : "true"}
>
  <path
    class="paper"
    d="M24 22A10 10 0 0 1 34 12H62A10 10 0 0 1 72 22V62L66 68 60 62 54 68 48 62 42 68 36 62 30 68 24 62Z"
  />
  <line class="rule" x1="33" y1="25" x2="63" y2="25" />

  {#if expression === "idle"}
    <circle class="dot" cx="39" cy="39" r="3.6" />
    <circle class="dot" cx="57" cy="39" r="3.6" />
    <path class="stroke" d="M41 50Q48 55 55 50" />
  {:else if expression === "pleased"}
    <path class="stroke" d="M35 40Q39 34.5 43 40" />
    <path class="stroke" d="M53 40Q57 34.5 61 40" />
    <path class="stroke" d="M40 48Q48 57 56 48" />
  {:else if expression === "worried"}
    <circle class="dot" cx="39" cy="38" r="3.2" />
    <circle class="dot" cx="57" cy="38" r="3.2" />
    <path class="stroke" d="M41 53Q48 47.5 55 53" />
  {:else if expression === "celebrating"}
    <path class="stroke" d="M35 38Q39 32.5 43 38" />
    <path class="stroke" d="M53 38Q57 32.5 61 38" />
    <path class="mouth" d="M40 47Q48 58 56 47Z" />
    <path class="spark-line" d="M18 18v8M14 22h8" />
    <path class="spark-line thin" d="M80 30v7M76.5 33.5h7" />
    <circle class="spark-dot" cx="78" cy="64" r="2.4" />
  {:else if expression === "waving"}
    <path class="stroke" d="M35 40Q39 34.5 43 40" />
    <path class="stroke" d="M53 40Q57 34.5 61 40" />
    <path class="stroke" d="M40 48Q48 57 56 48" />
    <path class="stroke" d="M71 46Q80 39 82 31" />
    <circle class="dot" cx="83" cy="27" r="4" />
    <path class="spark-line thin" d="M90 20q2 5 0 9" />
    <path class="spark-line thin" d="M86 34q4 -1 6 -4" />
  {:else if expression === "thinking"}
    <circle class="dot" cx="40" cy="36" r="3.4" />
    <circle class="dot" cx="58" cy="36" r="3.4" />
    <path class="stroke" d="M42 51Q48 48 54 52" />
    <circle class="spark-dot" cx="79" cy="30" r="2.2" />
    <circle class="spark-dot" cx="85" cy="23" r="3.2" />
  {:else}
    <path class="stroke" d="M35 39Q39 42.5 43 39" />
    <path class="stroke" d="M53 39Q57 42.5 61 39" />
    <ellipse class="dot" cx="48" cy="52" rx="2.4" ry="3" />
    <path class="spark-line thin" d="M74 30h7l-7 6h7" />
    <path class="spark-line thin small" d="M85 20h5l-5 4.5h5" />
  {/if}
</svg>

<style>
  .mascot {
    display: block;
  }

  .paper {
    fill: var(--paper-0);
    stroke: var(--ink-1);
    stroke-width: 3;
    stroke-linejoin: round;
  }

  .rule {
    stroke: var(--accent);
    stroke-width: 3;
    stroke-linecap: round;
  }

  .dot {
    fill: var(--ink-1);
  }

  .stroke {
    fill: none;
    stroke: var(--ink-1);
    stroke-width: 2.8;
    stroke-linecap: round;
  }

  .mouth {
    fill: var(--ink-1);
  }

  .spark-line {
    fill: none;
    stroke: var(--accent);
    stroke-width: 2.6;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  .spark-line.thin {
    stroke-width: 2.2;
  }

  .spark-line.small {
    stroke-width: 1.9;
  }

  .spark-dot {
    fill: var(--accent);
  }

  .bob {
    animation: bob 2.4s var(--ease-out) infinite;
  }

  @keyframes bob {
    0%,
    100% {
      transform: translateY(0);
    }
    50% {
      transform: translateY(-3px);
    }
  }
</style>
