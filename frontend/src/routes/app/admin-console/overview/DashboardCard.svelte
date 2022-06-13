<script lang="ts">
  import { fade } from 'svelte/transition';

  export let type: 'full' | 'half' | 'quarter' = 'full';
  export let important = false;
</script>

<div
  in:fade
  class="dashboard-card"
  class:dashboard-card--full={type === 'full'}
  class:dashboard-card--half={type === 'half'}
  class:dashboard-card--quarter={type === 'quarter'}
>
  <p class="t-uppercase t-micro t-color-text-3 dashboard-card__title">
    <slot name="title" />
  </p>
  <p
    class="dashboard-card__content"
    class:dashboard-card--important={important}
  >
    <slot name="value" />
  </p>
</div>

<style>
  .dashboard {
    margin: 60px 0;
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    grid-gap: 12px;
  }

  .dashboard-card {
    border-radius: 8px;
    padding: 12px;
    background: rgba(248, 250, 246, 0.03);
  }

  .dashboard-card__title {
    margin-bottom: 12px;
  }

  .dashboard-card__content {
    @mixin font medium;
    color: var(--color-text-5);
  }

  .dashboard-card--important {
    @mixin font xlarge;
    color: var(--color-primary);
  }

  .dashboard-card--full {
    grid-column: 1 / -1;
  }

  .dashboard-card--half {
    grid-column: span 2;
  }

  .dashboard-card--quarter {
    grid-column: span 1;
  }
</style>
