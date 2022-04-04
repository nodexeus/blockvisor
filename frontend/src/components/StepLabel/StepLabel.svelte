<script lang="ts">
  import CheckIcon from 'icons/check-empty-12.svg';

  export let step = 1;
  export let currentStep = 1;
  export let setStep;
  export let callback;

  const handleClick = () => {
    setStep(step);
    callback?.(step);
  };

  $: stepLabel =
    currentStep === step
      ? 'current'
      : currentStep > step
      ? 'completed'
      : 'disabled';

  $: stepClass = ['u-button-reset step t-label', `step--${stepLabel}`].join(
    ' ',
  );
</script>

<button
  class={stepClass}
  disabled={currentStep <= step}
  type="button"
  on:click={handleClick}
  {...$$restProps}
>
  <span class="step__number t-base">
    {#if currentStep > step}
      <CheckIcon />
    {:else}
      {step}
    {/if}
  </span>
  <span class="step__content">
    <slot />
  </span>
</button>

<style>
  .step {
    text-transform: uppercase;
    transition: color 0.25s var(--transition-easing-cubic);
    grid-gap: 12px;
    flex-wrap: wrap;
    display: grid;
    user-select: none;

    @media (--screen-large-max) {
      text-align: center;
    }

    @media (--screen-large) {
      align-items: center;
      grid-template-columns: 40px auto;
    }

    &__number {
      margin: 0 auto;
      width: 40px;
      aspect-ratio: 1;
      border-radius: 100%;
      display: inline-flex;
      align-items: center;
      justify-content: center;
      color: currentColor;
      border: 1px solid currentColor;
    }

    &__content {
      @media (--screen-large-max) {
        flex-basis: 100%;
      }
    }

    &--disabled {
      color: theme(--color-text-2);
      cursor: not-allowed;
    }

    &--current {
      color: theme(--color-text-4);
      cursor: not-allowed;
    }

    &--completed {
      color: theme(--color-primary);
    }
  }
</style>
