<script lang="ts">
  import { page } from '$app/stores';

  export let checkIfParent = false;
  export let urlAlias = '';

  $: isChildActive =
    checkIfParent && $page.url.pathname.includes(`${$$props.href}/`);

  $: isPartOfAlias = urlAlias && $page.url.pathname.includes(urlAlias);

  $: isActive =
    ($page.url.pathname.includes($$props.href) && !isChildActive) ||
    (isPartOfAlias && !isChildActive);
</script>

<a
  on:click
  href={$$props.ref}
  {...$$restProps}
  class={`nav-item ${$$props?.class ?? ''}`}
  class:nav-item--active={isActive}
>
  <span class="nav-item__icon" class:nav-item__icon--active={isActive}>
    <slot name="icon" />
  </span>
  <span class="nav-item__label">
    <slot name="label" />
  </span>
  <slot />
</a>

<style>
  .nav-item {
    display: flex;
    color: theme(--color-text-5);
    gap: 8px;
    border-radius: 4px;
    padding: 8px 12px;
    transition: background-color 0.18s var(--transition-easing-cubic);
    cursor: pointer;
    text-decoration: none;
    align-items: center;
    @mixin font tiny;

    &__label {
      white-space: nowrap;
      align-items: center;
      overflow: hidden;
      text-overflow: ellipsis;
      max-width: 100%;
      flex-grow: 1;
    }

    &__icon {
      min-width: 12px;
    }

    & :global(svg path) {
      color: theme(--color-text-2);
      fill: currentColor;
      transition: color 0.18s var(--transition-easing-cubic);
    }

    &__icon--active :global(svg path) {
      fill: theme(--color-primary);
    }

    &:not(.nav-item--active):hover,
    &:not(.nav-item--active):focus,
    &:not(.nav-item--active):active {
      & :global(svg path) {
        color: theme(--color-text-5);
      }
    }

    &--active {
      background-color: theme(--color-text-5-o10);
    }
  }
</style>
