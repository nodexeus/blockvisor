<script lang="ts">
  import anime from 'animejs';
  import { onMount } from 'svelte';

  export let id = 'js-value';
  export let value;

  const isNumericalValue = typeof value === 'number';

  const initialValue = isNumericalValue ? 0 : value;

  export let animeOptions = {
    delay: 100,
    duration: 800,
    round: 100,
  };

  onMount(() => {
    if (!isNumericalValue) {
      return;
    }

    anime({
      targets: `#${id} .title-with-value__field`,
      innerText: [0, value],
      easing: 'linear',
      ...animeOptions,
    });
  });
</script>

<div {id} class="title-with-value">
  <div class="t-uppercase t-microlabel title-with-value__title">
    <slot name="title" />
  </div>
  <div class="title-with-value__value">
    <output class="t-xlarge-fluid title-with-value__field"
      >{initialValue}</output
    >
    {#if $$slots.unit}
      <small class="t-label"><slot name="unit" /></small>
    {/if}
  </div>
</div>

<style>
  .title-with-value {
    white-space: nowrap;

    &__title {
      margin-bottom: 12px;
      color: theme(--color-text-4);
    }

    &__value {
      display: flex;
      align-items: baseline;
      gap: 8px;
    }
  }
</style>
