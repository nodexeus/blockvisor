<script lang="ts">
  import SubLinks from 'modules/app/components/Navigation/SubLinks.svelte';

  export let isActive;
  export let changeSubNav;
</script>

<div class="submenu" class:submenu--active={isActive}>
  <SubLinks {changeSubNav}>
    <slot />
  </SubLinks>
</div>

<style>
  .submenu {
    @media (--screen-medium-max) {
      display: none;
    }

    &--active {
      display: block;
    }

    @media (--screen-medium-large) {
      display: none;
    }
  }
</style>
