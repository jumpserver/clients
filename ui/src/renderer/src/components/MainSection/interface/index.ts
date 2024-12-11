export interface Platform {
  id: number;
  name: string;
}

export interface Connectivity {
  value: string;
  label: string;
}

export interface Nodes {
  id: string;
  name: string;
}

export interface Category {
  value: string;
  label: string;
}

export interface Type {
  value: string;
  label: string;
}

export interface IListItem {
  id: string;
  name: string;
  address: string;
  domain?: any;
  platform: Platform;
  org_id: string;
  connectivity: Connectivity;
  nodes: Nodes[];
  labels: any[];
  category: Category;
  type: Type;
  org_name: string;
  is_active: boolean;
  date_verified: string;
  date_created: string;
  comment: string;
  created_by: string;
}
