<article class="value-with-util">
  <div>
    <slot name="label" />
    <slot name="value" />
  </div>

  <div class="value-with-util__util">
    <slot name="util" />
  </div>
</article>

<style>
  .value-with-util {
    color: theme(--color-text-5);
    background-color: theme(--color-text-5-o3);
    display: flex;
    flex-direction: column;
    gap: 32px;
    border-radius: 4px;
    padding: 15px 16px;

    @media (--screen-medium-small) {
      justify-content: space-between;
      align-items: center;
      flex-direction: row;
    }
  }
</style>
