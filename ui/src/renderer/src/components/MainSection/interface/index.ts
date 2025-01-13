interface Platform {
  id: number;
  name: string;
}

interface Connectivity {
  value: string;
  label: string;
}

interface Nodes {
  id: string;
  name: string;
}

interface Category {
  value: string;
  label: string;
}

interface Type {
  value: string;
  label: string;
}

interface Actions {
  value: string;
  label: string;
}

interface Spec_info {}

interface Setting {
  old_ssh_version: boolean;
}

export interface ITypeObject {
  type?: string;
  category?: string;
}

export interface Permed_accounts {
  id: string;
  alias: string;
  name: string;
  username: string;
  has_username: boolean;
  has_secret: boolean;
  secret_type: string;
  actions: Actions[];
}

export interface Permed_protocols {
  name: string;
  port: number;
  public: boolean;
  setting: Setting;
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

export interface IItemDetail {
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
  spec_info: Spec_info;
  permed_protocols: Permed_protocols[];
  permed_accounts: Permed_accounts[];
  is_active: boolean;
  date_verified: string;
  date_created: string;
  comment: string;
  created_by: string;
}
