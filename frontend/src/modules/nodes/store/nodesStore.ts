import axios from 'axios';
import { ROUTES } from 'consts/routes';
import { NODES, USER_NODES } from 'modules/authentication/const';
import { writable } from 'svelte/store';

export const nodes = writable([]);
export const selectedNode = writable({});

const groupBy = (array, key) => {
  return array.reduce((result, currentValue) => {
    (result[currentValue[key]] = result[currentValue[key]] || []).push(
      currentValue,
    );
    return result;
  }, {});
};

const token =
  'eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzUxMiJ9.eyJzdWIiOiI4OWI3MzgyMi04ODM3LTQ5NTAtOTA4Yy0zZTNiM2E4MjJlMzQiLCJyb2xlIjoiYWRtaW4iLCJleHAiOjE2NTE4NDA0MTV9.i_TPUQ7kN8mXJ5i793q3BcvcbYP_n_oNWU-OHjujzl4I0oxEIDbsNEHqcnJVm6sPZTgOV3SUHM-TjAqcWMNsdw';

export const fetchAllNodes = async () => {
  let all_nodes = [
    {
      title: 'All Nodes',
      href: ROUTES.NODES,
      children: [],
    },
  ];

  const res = await axios.get(NODES, {
    headers: { Authorization: `Bearer ${token}` },
  });

  const filteredData = res.data.slice(0, 100);

  const groupedData = Object.entries(groupBy(filteredData, 'user_id'));

  all_nodes[0].children = groupedData.map((item) => {
    return {
      title: item[0],
      href: ROUTES.NODE_GROUP(item[0]),
      id: 'host-id1',
    };
  });

  nodes.set(all_nodes);
};

export const fetchNodeById = async (id: string) => {
  const res = await axios.get(USER_NODES(id), {
    headers: { Authorization: `Bearer ${token}` },
  });

  selectedNode.set(res.data);
};
