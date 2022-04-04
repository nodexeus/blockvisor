<script lang="ts">
  import DataState from 'components/DataState/DataState.svelte';

  export let state: HostState | NodeState;

  const parentClass = [
    'data',
    `data--${!!$$slots.icon ? 'with-icon' : 'no-icon'}`,
    `data--${state}`,
  ].join(' ');
</script>

<div class={parentClass}>
  <div class="data__content">
    {#if $$slots.icon}
      <p class="data__icon"><slot name="icon" /></p>
    {/if}
    <p class="data__primary"><slot name="primary" /></p>
    <p class="data__secondary"><slot name="secondary" /></p>
  </div>
  <div class="data__state">
    <div class="t-uppercase data__label">
      <DataState {state} />
    </div>
    <slot name="action" />
  </div>
</div>

<style>
  .data {
    display: grid;
    grid-gap: 16px;
    padding-top: 28px;
    padding-bottom: 10px;
    border-bottom: 1px solid theme(--color-text-5-o10);

    @media (--screen-medium-small) {
      grid-template-columns: auto 175px;
      grid-gap: 44px;
      align-items: flex-start;
    }

    & :global(.dropdown) {
      margin-top: 0;
    }

    &__label {
      @mixin font microlabel;
    }

    &__state {
      justify-content: space-between;
      color: theme(--color-text-3);
      display: flex;
      gap: 30px;
      align-items: center;
    }

    &__content {
      display: flex;
      flex-direction: column;
      gap: 8px;
    }

    &__primary {
      display: flex;
      flex-wrap: wrap;
      gap: 8px;
      color: theme(--color-text-5);
      align-items: baseline;

      & > :global(*:first-child) {
        flex-grow: 1;

        @media (--screen-medium-small) {
          flex-grow: 0;
        }
      }

      & :global(small) {
        @mixin font small;
        color: theme(--color-text-3);
        max-width: 92px;
        text-overflow: ellipsis;
        overflow: hidden;
        white-space: nowrap;
      }
    }

    &__secondary {
      @mixin font small;
      flex-wrap: wrap;
      display: flex;
      align-items: baseline;
      gap: 12px;
      color: theme(--color-text-2);
    }

    &__icon {
      color: theme(--color-text-2);
    }

    &--with-icon {
      .data__content {
        @media (--screen-medium-small) {
          display: grid;
          grid-column-gap: 16px;
          grid-template-rows: auto auto;
          grid-template-columns: 24px auto;
        }
      }

      .data__secondary {
        grid-column-start: 2;
      }
    }

    &--issue {
      .data__primary,
      .data__state {
        color: var(--color-utility-warning);
      }

      :global(.data-state) {
        & :global(svg) {
          backface-visibility: hidden;
          animation: blink 1s var(--transition-easing-cubic) infinite alternate;
        }
      }
    }

    &--consensus {
      background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 512 1'%3E%3Cpath stroke='url(%23a)' d='M0 .5h512'/%3E%3Cdefs%3E%3ClinearGradient id='a' x1='512' x2='0' y1='1' y2='1' gradientUnits='userSpaceOnUse'%3E%3Cstop stop-color='%23BFF589' stop-opacity='0'/%3E%3Cstop offset='.5' stop-color='%23BFF589'/%3E%3Cstop offset='1' stop-color='%23BFF589' stop-opacity='0'/%3E%3C/linearGradient%3E%3C/defs%3E%3C/svg%3E");
      border-width: 0;
      background-position-y: 100%;
      background-repeat: no-repeat;

      & :global(.data-state),
      .data__primary {
        color: var(--color-primary);
      }

      .data__icon {
        color: var(--color-primary);
      }

      :global(.data-state) {
        & :global(svg) {
          backface-visibility: hidden;
          animation: rotateClockwise 2s linear infinite reverse;
        }
      }
    }
  }
</style>
