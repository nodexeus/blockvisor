<script lang="ts">
  import type { PaginationProps } from './Pagination.svelte';

  export let pages = 10;
  export let perPage = 4;
  export let itemsTotal = 22;
  export let currentPage = 1;
  export let onPageChange: PaginationProps['onPageChange'];

  $: rangeMin = currentPage * perPage - perPage + 1;
  $: rangeMax =
    currentPage * perPage > itemsTotal ? itemsTotal : currentPage * perPage;

  $: isFirstPage = currentPage === 1;
  $: isLastPage = currentPage === pages;

  const handleClick = (e) => {
    const page = parseInt(e.currentTarget.dataset.page);
    if (currentPage === page || isNaN(page)) {
      return;
    }

    onPageChange(page);
  };
</script>

<nav class="pagination">
  <div class="pagination__controls">
    <button
      on:click={handleClick}
      data-page={currentPage - 1}
      class="u-button-reset pagination__button pagination__button--control"
      disabled={isFirstPage}>Previous</button
    >
    <div class="pagination__wrapper">
      <div
        class="pagination__indicator"
        style={`transform: translate3d(${(currentPage - 1) * 100}%, 0,0 )`}
      />
      <ul class="u-list-reset pagination__pages">
        {#each [...new Array(pages)] as _, index}
          <li>
            <button
              on:click={handleClick}
              data-page={index + 1}
              class="u-button-reset pagination__button pagination__button--page"
              class:pagination__button--page-active={index + 1 === currentPage}
              >{index + 1}</button
            >
          </li>
        {/each}
      </ul>
    </div>
    <button
      on:click={handleClick}
      data-page={currentPage + 1}
      class="u-button-reset pagination__button pagination__button--control"
      disabled={isLastPage}>Next</button
    >
  </div>

  <div class="u-o-40">{rangeMin} - {rangeMax} of {itemsTotal}</div>
</nav>

<style>
  .pagination {
    color: theme(--color-text-5);
    display: flex;
    gap: 16px 80px;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
    isolation: isolate;
  }

  .pagination__controls {
    overflow: auto;
    display: flex;
    gap: 12px;
  }

  .pagination__wrapper {
    position: relative;
  }

  .pagination__pages {
    display: flex;
  }

  .pagination__indicator {
    backface-visibility: hidden;
    border-radius: 50px;
    z-index: var(--level-1);
    width: 39px;
    border: 1px solid theme(--color-primary);
    height: 100%;
    position: absolute;
    left: 0;
    top: 0;
    transition: transform 0.3s ease-in-out;
  }

  .pagination__button {
    display: inline-flex;
    text-align: center;
    justify-content: center;
    align-items: center;
    @mixin font small;
  }

  .pagination__button--control {
    color: theme(--color-text-5);
    transition: opacity 0.18s var(--transition-easing-cubic),
      box-shadow 0s linear;
    box-shadow: 0 0 0 0 currentColor;
  }

  .pagination__button--control:not(:disabled):hover,
  .pagination__button--control:not(:disabled):active {
    box-shadow: 0 1px 0 0 currentColor;
  }

  .pagination__button--control:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .pagination__button--page {
    height: 28px;
    width: 39px;
    color: theme(--color-text-5);
    opacity: 0.4;
    border-radius: 50px;
    transition: color 0.2s var(--transition-easing-cubic),
      opacity 0.2s var(--transition-easing-cubic),
      background-color 0.2s var(--transition-easing-cubic);
  }

  .pagination__button--page:not(.pagination__button--page-active):hover,
  .pagination__button--page:not(.pagination__button--page-active):focus,
  .pagination__button--page:not(.pagination__button--page-active):active {
    opacity: 0.8;
    background-color: theme(--color-input-background);
  }

  .pagination__button--page-active {
    opacity: 1;
    color: theme(--color-primary);
  }
</style>
