import * as React from 'react';
import { PostgrestClient, PostgrestSingleResponse } from '@supabase/postgrest-js'

interface Props {
    postgrest: PostgrestClient;
}

interface Member {
    email: string;
    first_name: string;
    last_name: string;
    occupations: [Occupation];
}

interface Occupation {
    company_name: string | null;
    position: string | null;
}

interface State {
    data: Member[] | null; // TODO: fix me
    isLoading: boolean;
}

const TableRow = (member: Member) => {
    return (
        <tr>
            <td>{member.email}</td>
            <td>{member.first_name}</td>
            <td>{member.last_name}</td>
            <td>{member.occupations.length > 0 ? member.occupations[0].company_name : null}</td>
        </tr>
    )
}

const MembersTable = (props: Props) => {
    const [state, setState] = React.useState<State>({
        data: null,
        isLoading: false,
    });

    const columns = ['email', 'first_name', 'last_name', 'occupations(company_name, position)'];
    React.useEffect(() => {
        if (state.data !== null || state.isLoading) {
            return;
        }
        setState({ ...state, isLoading: true });
        props.postgrest.from('members').select(columns.join(','))
            .then((result: PostgrestSingleResponse<any[]>) => setState({ ...state, data: result.data, isLoading: false }))
    });

    return (
        <div>
            <div>Table</div>
            <table>
                <thead>
                    <tr>
                        {columns.map((v) => {
                            return (
                                <th>{v}</th>
                            )
                        })}
                    </tr>
                </thead>
                <tbody>
                    {(state.data || []).map(TableRow)}
                </tbody>
            </table>
        </div>
    )
}

export default MembersTable;
