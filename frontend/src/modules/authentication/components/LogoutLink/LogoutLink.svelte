<script>
  import axios from 'axios';

  import { ROUTES } from 'consts/routes';

  const handleClick = async () => {
    axios.post(ROUTES.AUTH_LOGOUT).then(() => {
      location.reload();
    });
  };
</script>

<a
  class="link link--primary"
  href="#logout"
  on:click|preventDefault={handleClick}
  {...$$restProps}><slot /></a
>
