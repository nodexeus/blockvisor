<script lang="ts">
  import { app } from 'modules/app/store';

  import IconRight from 'icons/arrow-small-4.svg';

  $: breadcrumbs = $app.breadcrumbs;
</script>

<ul class="u-list-reset breadcrumbs">
  {#each breadcrumbs as breadcrumb, index}
    {#if index === breadcrumbs.length - 1}
      <li class="breadcrumbs__item breadcrumbs__item--last">
        <span class="breadcrumbs__link breadcrumbs__link--last"
          >{breadcrumb.title}</span
        >
      </li>
    {:else}
      <li class="breadcrumbs__item">
        <a href={breadcrumb.url} class="link link--primary breadcrumbs__link"
          >{breadcrumb.title}</a
        >
        <IconRight class="dashboard__icon" />
      </li>
    {/if}
  {/each}
</ul>

<style>
  .breadcrumbs {
    display: inline-flex;

    &__item {
      display: inline-flex;
      align-items: center;
    }

    & :global(svg) {
      min-width: 8px;
      margin: 0 8px;
      opacity: 0.2;
    }
  }
</style>
