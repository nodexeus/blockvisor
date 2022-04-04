<script lang="ts">
  import IconArrow from 'icons/arrow-right-12.svg';

  export let title;
  export let description;
</script>

<a href="#TODO" class="u-link-reset group-item">
  <div class="group-item__icon">
    <slot name="icon" />
  </div>
  <div class="group-item__content">
    <span class="group-item__title t-small">{title}</span>
    <span class="group-item__description t-microlabel t-uppercase"
      >{description}</span
    >
  </div>
  <IconArrow class="group-item__action" />
</a>

<style>
  .group-item {
    display: grid;
    grid-template-columns: 12px auto 12px;
    grid-gap: 12px;

    &__title,
    &__description {
      display: block;
    }

    &__description {
      color: theme(--color-text-3);
    }

    & :global(&__icon) {
      margin-top: 4px;
      color: theme(--color-text-2);
    }

    & :global(&__action) {
      align-self: center;
      color: theme(--color-text-2);
    }
  }
</style>
