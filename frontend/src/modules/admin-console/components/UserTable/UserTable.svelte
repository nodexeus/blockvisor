<script lang="ts">
  import Sorter from 'components/Sorter/Sorter.svelte';
  import UserDataRow from './UserDataRow.svelte';

  let sortActive;

  const handleSort = (id: string, value: SorterValues) => {
    sortActive = { id, value };
  };
</script>

<table class="table user-table t-small">
  <colgroup>
    <col width="155px" />
    <col width="230px" />
    <col width="200px" />
    <col />
    <col width="44px" />
  </colgroup>
  <thead class="user-table__head t-colo">
    <tr>
      <th class="table__heading">
        <Sorter callback={handleSort} active={sortActive} id="first-name"
          >Full name</Sorter
        >
      </th>
      <th class="table__heading">
        <Sorter callback={handleSort} active={sortActive} id="e-mail"
          >E-mail</Sorter
        >
      </th>
      <th class="table__heading">
        <Sorter callback={handleSort} active={sortActive} id="organization"
          >Organization Name</Sorter
        >
      </th>
      <th class="table__heading">
        <span class="visually-hidden">Status</span>
      </th>
      <th class="table__heading">
        <span class="visually-hidden">Actions</span>
      </th>
    </tr>
  </thead>
  <tbody class="user-table__body">
    <UserDataRow />
    <UserDataRow />
    <UserDataRow />
    <UserDataRow />
    <UserDataRow />
    <UserDataRow />
  </tbody>
</table>

<style>
  .user-table {
    margin-bottom: 80px;

    @media (--screen-large-max) {
      display: block;
    }

    &__head {
      @media (--screen-large-max) {
        display: none;
      }
    }

    &__body {
      @media (--screen-large-max) {
        display: block;
      }
    }
  }
</style>
