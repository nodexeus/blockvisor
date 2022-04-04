<script lang="ts">
  import IconCheckmark from 'icons/checkmark-12.svg';

  export let highlight = false;
  export let plan = [];

  $: classes = ['plan-card', `plan-card--${highlight ? 'highlight' : ''}`].join(
    ' ',
  );
</script>

<article class={classes}>
  <div class="plan-card__info">
    <header class="plan-card__header">
      <div class="t-uppercase t-label plan-card__title">
        <slot name="title" />
      </div>

      <slot name="featured" />
    </header>

    <div class="plan-card__price">
      <span class="t-xlarge plan-card__fee">
        <slot name="fee" />
      </span>
      <small class="t-microlabel t-uppercase plan-card__description">
        <slot name="description" />
      </small>
    </div>
    {#if $$slots.note}
      <small class="t-color-text-4 t-tiny plan-card__note">
        <slot name="note" />
      </small>
    {/if}

    {#if Boolean(plan.length)}
      <ul class="t-small u-list-reset plan-card__features">
        {#each plan as item}
          <li class="plan-card__features-item">
            <IconCheckmark />
            {item}
          </li>
        {/each}
      </ul>
    {/if}
  </div>
  <div class="plan-card__action">
    <slot name="action" />
  </div>
</article>

<style>
  .plan-card {
    display: flex;
    min-height: 100%;
    flex-direction: column;
    background-color: theme(--color-text-5-o3);
    padding: 20px;
    gap: 40px;

    &__note {
      display: block;
      margin-bottom: 32px;
    }

    &__header {
      display: flex;
      flex-wrap: wrap;
      justify-content: space-between;
      gap: 12px 40px;
      margin-bottom: 40px;
    }

    &--highlight {
      background-color: theme(--color-overlay-background-1);
    }

    &__features-item {
      display: flex;
      gap: 12px;
      padding: 12px 0;
      border-top: 1px solid theme(--color-text-5-o10);

      & :global(svg) {
        flex: 0 0 12px;
        margin-top: 4px;
        color: theme(--color-text-2);
      }
    }

    &__price {
      display: flex;
      gap: 12px;
      align-items: flex-end;
      margin-bottom: 20px;
    }

    &__description {
      color: theme(--color-text-3);
    }

    &__info {
      flex-grow: 1;
    }
  }
</style>
