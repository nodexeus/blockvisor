<script lang="ts">
  export let isActive;
</script>

<div class="parent-menu" class:parent-menu--active={isActive}>
  <slot />
</div>

<style>
  .parent-menu {
    @media (--screen-medium-max) {
      display: none;

      &--active {
        display: block;
      }
    }
  }
</style>
