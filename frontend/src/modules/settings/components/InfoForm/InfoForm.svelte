<script lang="ts">
  import type { UiContainer, UiNodeInputAttributes } from '@ory/kratos-client';
  import Button from 'components/Button/Button.svelte';
  import Input from 'modules/forms/components/Input/Input.svelte';
  import { email, Hint, required, useForm } from 'svelte-use-form';

  export let ui: UiContainer;

  let fields = ui.nodes.reduce((acc, node) => {
    const { name, value } = node.attributes as UiNodeInputAttributes;
    acc[name] = {
      value: value || '',
    };
    return acc;
  }, {});

  const form = useForm({
    'traits.email': {
      initial: fields['traits.email'].value,
    },
    'traits.name.first': {
      initial: fields['traits.name.first'].value,
    },
    'traits.name.last': {
      initial: fields['traits.name.last'].value,
    },
  });
</script>

<form
  action={ui.action}
  method={ui.method}
  enctype="application/x-www-form-urlencoded"
  use:form
>
  <ul class="u-list-reset">
    <li class="visually-hidden">
      <input
        bind:value={fields['csrf_token'].value}
        type="hidden"
        name="csrf_token"
      />
    </li>

    <li>
      <Input
        field={$form['traits.email']}
        name="traits.email"
        validate={[required, email]}
      >
        <svelte:fragment slot="label">Email</svelte:fragment>
        <svelte:fragment slot="hints">
          <Hint on="required">This is a mandatory field</Hint>
          <Hint on="email" hideWhenRequired>Email is not valid</Hint>
        </svelte:fragment>
      </Input>
    </li>

    <li>
      <Input
        field={$form['traits.name.first']}
        name="traits.name.first"
        validate={[required]}
      >
        <svelte:fragment slot="label">First Name</svelte:fragment>
        <svelte:fragment slot="hints">
          <Hint on="required">This is a mandatory field</Hint>
        </svelte:fragment>
      </Input>
    </li>
    <li>
      <Input
        field={$form['traits.name.last']}
        name="traits.name.last"
        validate={[required]}
      >
        <svelte:fragment slot="label">Last Name</svelte:fragment>
        <svelte:fragment slot="hints">
          <Hint on="required">This is a mandatory field</Hint>
        </svelte:fragment>
      </Input>
    </li>
    <li>
      <Button
        style="secondary"
        size="medium"
        type="submit"
        name="method"
        value="profile"
      >
        Save
      </Button>
    </li>
  </ul>
</form>
