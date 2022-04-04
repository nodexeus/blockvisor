<tr class="node-data-row">
  <th
    class="t-uppercase t-microlabel t-color-text-3 node-data-row__col node-data-row__col--heading"
  >
    <slot name="label" /></th
  >
  <td class="node-data-row__col"><slot /></td>
</tr>

<style>
  .node-data-row {
    word-break: break-all;
    border-bottom: 1px solid theme(--color-text-5-o10);

    &__col {
      padding-top: 28px;
      padding-bottom: 8px;

      @media (--screen-medium-larger-max) {
        padding-top: 8px;
        padding-bottom: 8px;
        display: block;
      }

      &--heading {
        padding-right: 28px;
        font-weight: var(--font-weight-normal);

        @media (--screen-medium-larger-max) {
          padding-top: 28px;
          padding-bottom: 0;
        }
      }
    }

    @media (--screen-medium-larger-max) {
      display: block;
    }
  }
</style>
