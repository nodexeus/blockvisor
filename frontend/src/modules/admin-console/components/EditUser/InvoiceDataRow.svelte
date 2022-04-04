<script lang="ts">
  import IconEye from 'icons/eye-12.svg';
  import IconDownload from 'icons/download-12.svg';
  import { ROUTES } from 'consts/routes';
</script>

<tr class="table__row invoice-row">
  <td class="invoice-row__col"> 00003/2022 </td>
  <td class="invoice-row__col"> 04/24/2022 </td>
  <td class="invoice-row__col invoice-row__col--actions">
    <a
      class="link link--icon invoice-row__link"
      href={ROUTES.ADMIN_CONSOLE_INVOICE('example-id', 'invoice-id')}
    >
      <IconEye />
    </a>

    <a
      class="link link--icon invoice-row__link"
      href={`#TODO`}
      target="_blank"
      rel="noopener noreferrer"
    >
      <IconDownload />
    </a>
  </td>
</tr>

<style>
  .invoice-row {
    @media (--screen-smaller-max) {
      display: block;
      position: relative;
    }

    &__col {
      padding-top: 32px;
      padding-bottom: 18px;
      padding-right: 28px;

      @media (--screen-smaller-max) {
        padding: 16px 72px 16px 0;
        display: block;

        &--actions {
          position: absolute;
          top: 0;
          right: 0;
        }
      }

      &:last-child {
        padding-right: 0;
      }
    }

    &__link {
      display: inline-block;
      padding: 2px 8px;
    }
  }
</style>
