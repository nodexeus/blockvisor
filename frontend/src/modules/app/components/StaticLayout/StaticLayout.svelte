<div class="static-layout">
  {#if $$slots.title || $$slots.util}
    <header class="static-layout__header">
      <slot name="title" />
      <slot name="util" />
    </header>
  {/if}
  <section>
    <slot />
  </section>
</div>

<style>
  .static-layout {
    margin-top: 40px;
    grid-column: 1/13;

    @media (--screen-large) {
      margin-top: 80px;
      grid-column: 2/12;
    }

    &__header {
      margin-bottom: 40px;
      flex-wrap: wrap;
      display: flex;
      gap: 12px 40px;
      flex-direction: column;

      @media (--screen-medium-small) {
        flex-direction: row;
        justify-content: space-between;
        align-items: baseline;
      }

      @media (--screen-large) {
        margin-bottom: 80px;
      }
    }
  }
</style>
