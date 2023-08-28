import * as React from "react";
import {
  PostgrestClient,
  PostgrestSingleResponse,
} from "@supabase/postgrest-js";
import { DataGrid, GridColDef, GridValueGetterParams } from "@mui/x-data-grid";
import Box from "@mui/material/Box";

interface Props {
  postgrest: PostgrestClient;
}

interface Member {
  member_number: number;
  id: string;
  email: string;
  first_name: string;
  last_name: string;
  phone_number: string;
}

interface Occupation {
  company_name: string | null;
  position: string | null;
}

interface State {
  data: Member[] | [];
  isLoading: boolean;
  initialDataLoaded: boolean;
}

const MembersTable = (props: Props) => {
  const [state, setState] = React.useState<State>({
    data: [],
    isLoading: false,
    initialDataLoaded: false,
  });

  const columns = [
    "member_number",
    "id",
    "email",
    "first_name",
    "last_name",
    "phone_number",
  ];
  React.useEffect(() => {
    if (state.initialDataLoaded !== false || state.isLoading) {
      return;
    }
    setState({ ...state, isLoading: true });
    props.postgrest
      .from("members")
      .select(columns.join(","))
      .then((result: PostgrestSingleResponse<any[]>) => {
        if (result.data !== null) {
          setState({
            ...state,
            initialDataLoaded: true,
            data: result.data,
            isLoading: false,
          });
        } else {
          setState({ ...state, isLoading: false });
        }
      });
  });

  const gridcolumns: GridColDef[] = [
    {
      field: "member_number",
      headerName: "Member Number",
      headerClassName: "table-header", //TODO: refactor so the classname is generated
      description: "Member number",
      flex: 0.5,
      minWidth: 120,
    },
    {
      field: "last_name",
      headerName: "Last name",
      headerClassName: "table-header",
      flex: 1.5,
      minWidth: 150,
    },
    {
      field: "first_name",
      headerName: "First Name",
      headerClassName: "table-header",
      flex: 1.5,
      minWidth: 150,
    },
    {
      field: "phone_number",
      headerName: "Phone number",
      headerClassName: "table-header",
      flex: 1.0,
      minWidth: 150,
    },
    {
      field: "email",
      headerName: "Email",
      headerClassName: "table-header",
      flex: 1.5,
      minWidth: 150,
    },
  ];

  return (
    <div>
      <h1>Members Table</h1>
      <Box sx={{ height: "100%", width: "100%" }}>
        <DataGrid
          rows={state.data}
          columns={gridcolumns}
          initialState={{
            pagination: {
              paginationModel: {
                pageSize: 100,
              },
            },
          }}
          pageSizeOptions={[100, 50, 25]}
          disableRowSelectionOnClick
        />
      </Box>
    </div>
  );
};

export default MembersTable;
