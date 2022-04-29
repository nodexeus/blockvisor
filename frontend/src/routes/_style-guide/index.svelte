<script lang="ts">
  import IconCog from 'icons/close-12.svg';
  import IconSearch from 'icons/search-16.svg';
  import IconUser from 'icons/contact-16.svg';
  import Button from 'components/Button/Button.svelte';
  import Input from 'modules/forms/components/Input/Input.svelte';
  import Textarea from 'modules/forms/components/Textarea/Textarea.svelte';
  import Select from 'modules/forms/components/Select/Select.svelte';
  import { useForm, Hint, required } from 'svelte-use-form';
  import Pagination from 'modules/app/components/Pagination/Pagination.svelte';
  import Avatar from 'components/Avatar/Avatar.svelte';
  import Checkbox from 'modules/forms/components/Checkbox/Checkbox.svelte';
  import PrivateRoute from 'modules/authorization/components/PrivateRoute/PrivateRoute.svelte';
  import TagsField from 'modules/forms/components/TagsField/TagsField.svelte';
  import StatsWithState from 'components/StatsWithState/StatsWithState.svelte';
  import ValueWithUtil from 'modules/forms/components/ValueWithUtil/ValueWithUtil.svelte';
  import FileUpload from 'modules/forms/components/FileUpload/FileUpload.svelte';
  import MultiSelect from 'modules/forms/components/MultiSelect/MultiSelect.svelte';
  import Toggle from 'modules/forms/components/Toggle/Toggle.svelte';
  import TabFirst from './_TabFirst.svelte';
  import TabSecond from './_TabSecond.svelte';
  import TabThird from './_TabThird.svelte';
  import Tabs from 'modules/app/components/Tabs/Tabs.svelte';
  import ConfirmDeleteModal from 'modules/app/components/ConfirmDeleteModal/ConfirmDeleteModal.svelte';

  let currentPage = 1;

  const form = useForm({
    sweets: { initial: 'chocolate' },
  });

  let files = {
    accepted: [],
    rejected: [],
  };

  function handleFilesSelect(e) {
    e.preventDefault();
    e.stopPropagation();

    const { acceptedFiles, fileRejections } = e.detail;
    files.accepted = acceptedFiles;
    files.rejected = fileRejections;
  }

  function handleFilesRemove(e) {
    e.preventDefault();
    e.stopPropagation();

    files = {
      accepted: [],
      rejected: [],
    };
  }

  const handleOnClick = () => {
    document.documentElement.classList.toggle('theme--light');
    document.documentElement.classList.toggle('theme--dark');
  };

  let items = [
    { label: 'Really long tab title', value: 1, component: TabFirst },
    { label: 'Another really long title', value: 2, component: TabSecond },
    {
      label: 'This is now really really long',
      value: 3,
      component: TabThird,
    },
  ];
</script>

<PrivateRoute>
  <ValueWithUtil id="node-type">
    <svelte:fragment slot="label">Node type</svelte:fragment>
    <svelte:fragment slot="value">Node/api</svelte:fragment>
    <Button slot="util" size="small" style="outline">Change</Button>
  </ValueWithUtil>

  <Tabs {items} />

  <ConfirmDeleteModal
    on:submit={(e) => {
      e.preventDefault();
    }}
    isModalOpen
    handleModalClose={() => {}}
  >
    <svelte:fragment slot="label">
      Type “DELETE” to confirm deletion of “HostFox” host.
    </svelte:fragment>
  </ConfirmDeleteModal>

  <section class="component">
    <StatsWithState id="js-test" state="active" value={50}>
      <IconCog />
      <span class="t-uppercase"> Nodes </span>
    </StatsWithState>
    <Avatar size="medium" src={'https://picsum.photos/400/400'} />
    <Pagination
      pages={6}
      perPage={4}
      itemsTotal={22}
      {currentPage}
      onPageChange={(page) => {
        currentPage = page;
      }}
    />

    <form
      use:form
      on:submit={(e) => {
        e.preventDefault();
        $form.test.touched = true;
        console.log($form);
      }}
    >
      <Toggle field={$form?.testToggle} name="testToggle">
        This is a toggle

        <svelte:fragment slot="description"
          >This is a description</svelte:fragment
        >
      </Toggle>
      <MultiSelect validate={[required]} field={$form.test} name="test">
        <svelte:fragment slot="label">Group</svelte:fragment>
        <svelte:fragment slot="hints">
          <Hint on="required">This is a mandatory field</Hint>
        </svelte:fragment>
      </MultiSelect>
      <TagsField field={$form?.tags} name="tags">
        <svelte:fragment slot="label">Tags</svelte:fragment>
      </TagsField>
      <FileUpload
        files={files.accepted}
        on:click={handleFilesRemove}
        on:drop={handleFilesSelect}>Upload an existing swarm key</FileUpload
      >
      <Select
        items={[
          { value: 'chocolate', label: 'Chocolate' },
          { value: 'pizza', label: 'Pizza' },
          { value: 'cake', label: 'Cake' },
          { value: 'chips', label: 'Chips' },
          { value: 'ice-cream', label: 'Ice Cream' },
        ]}
        size="small"
        field={$form?.sweets}
        name="sweets"
        description="Some description"
      >
        <IconUser slot="utilLeft" />
        <svelte:fragment slot="label">Input label</svelte:fragment>
        <svelte:fragment slot="hints">
          <Hint on="required">This is a mandatory field</Hint>
        </svelte:fragment>
      </Select>
      <Input
        name="name"
        field={$form?.name}
        description="Some description"
        placeholder="Password label"
      >
        <IconUser slot="utilLeft" />
        <svelte:fragment slot="label">Input label</svelte:fragment>
        <svelte:fragment slot="hints">
          <Hint on="required">This is a mandatory field</Hint>
          <Hint on="email" hideWhenRequired>Email is not valid</Hint>
        </svelte:fragment>
      </Input>
      <Checkbox
        name="accept"
        field={$form?.accept}
        description="Checkbox description"
      >
        Check me out
        <svelte:fragment slot="hints">
          <Hint on="required">This is a mandatory field</Hint>
        </svelte:fragment>
      </Checkbox>
      <Input
        field={$form?.password}
        name="password"
        description="Some description"
        placeholder="Password label"
      >
        <IconUser slot="utilRight" />
        <svelte:fragment slot="label">Password label</svelte:fragment>
        <svelte:fragment slot="hints">
          <Hint on="required">This is a mandatory field</Hint>
          <Hint on="email" hideWhenRequired>Email is not valid</Hint>
        </svelte:fragment>
      </Input>
      <Textarea
        field={$form?.desc}
        name="desc"
        description="Some description"
        placeholder="Description label"
      >
        <IconUser slot="utilRight" />
        <svelte:fragment slot="label">Description label</svelte:fragment>
        <svelte:fragment slot="hints">
          <Hint on="required">This is a mandatory field</Hint>
        </svelte:fragment>
      </Textarea>
      <Button size="medium" style="secondary" type="submit">Submit</Button>
    </form>
    <p class="t-micro component__child">I inherit the theme text color</p>
    <p class="t-tiny component__child">I inherit the theme text color</p>
    <p class="t-small component__child">I inherit the theme text color</p>
    <p class="t-base component__child">I inherit the theme text color</p>
    <p class="t-medium component__child">I inherit the theme text color</p>
    <p class="t-large component__child">I inherit the theme text color</p>
    <p class="t-xlarge component__child">I inherit the theme text color</p>
    <p class="t-xxlarge component__child component__child--light ">
      I have a special color on a light theme
    </p>
    <p class="t-xxxlarge component__child component__child--light-alt">
      I have a special color on a light theme
    </p>
    <p class="t-huge component__child component__child--dark">
      I have a special color on a dark theme
    </p>
  </section>

  <section class="component">
    <p class="t-base-fluid component__child">
      I have a fluid typography value! Resize the screen!
    </p>
    <p class="t-medium-fluid component__child">
      I have a fluid typography value! Resize the screen!
    </p>
    <p class="t-large-fluid component__child">
      I have a fluid typography value! Resize the screen!
    </p>
    <p class="t-xlarge-fluid component__child">
      I have a fluid typography value! Resize the screen!
    </p>
    <p class="t-xxlarge-fluid component__child">
      I have a fluid typography value! Resize the screen!
    </p>
    <p class="t-xxxlarge-fluid component__child">
      I have a fluid typography value! Resize the screen!
    </p>
    <p class="t-huge-fluid component__child">
      I have a fluid typography value! Resize the screen!
    </p>
  </section>

  <Button
    title="View article"
    border="rounded"
    size="medium"
    style="primary"
    on:click={handleOnClick}
  >
    <svelte:fragment>
      <IconSearch />
      Toggle theme
    </svelte:fragment>
  </Button>
</PrivateRoute>

<style lang="postcss">
  /* All colors must be added via theme so we an easily switch between color schemes and override them */
  .component {
    padding: 2rem;
    background-color: theme(--color-foreground-secondary);
    overflow: hidden;
  }

  /* Text color automatically switches as theme switches */
  .component__child {
    color: theme(--color-text-3, --component-child-color);
    margin: 1rem 0;
  }

  /* This is how we add overrides for dark theme */
  @mixin theme dark {
    & .component__child--dark {
      --component-child-color: var(--color-primary);
    }
  }

  /* This is how we add overrides for light theme */
  @mixin theme light {
    & .component__child--light {
      --component-child-color: var(--color-utility-warning);
    }

    & .component__child--light-alt {
      --component-child-color: var(--color-utility-note);
    }
  }
</style>
