<ul class="u-list-reset steps">
  <slot />
</ul>

<style>
  .steps {
    display: inline-flex;
    max-width: 100%;
    overflow-x: auto;
    gap: 16px;
    white-space: nowrap;

    > :global(li) {
      position: relative;
    }

    > :global(li + li::before) {
      content: '';
      position: absolute;
      background-color: theme(--color-text-5-o10);

      @media (--screen-large-max) {
        width: 80px;
        height: 1px;
        top: 25%;
        left: -16px;
        transform: translate3d(-50%, 0, 0);
      }

      @media (--screen-large) {
        height: 20px;
        width: 1px;
        bottom: calc(100% + 16px);
        left: 20px;
        gap: 52px;
        flex-direction: column;
      }
    }

    @media (--screen-large-max) {
      gap: 32px;
    }

    @media (--screen-large) {
      gap: 52px;
      flex-direction: column;
    }
  }
</style>
