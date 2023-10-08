import * as React from 'react';

import { useParams } from 'react-router-dom';


import { PostgrestClient, PostgrestSingleResponse } from '@supabase/postgrest-js'


interface Member {
    member_number: Number | null;
    email: string | null;
    first_name: string | null;
    last_name: string | null;
    date_of_birth: Date | null;
    address: string | null;
    city: String | null;
    postal_code: String | null;
    language: String;
    phone_number: String | null;
}

interface Occupation {
	company_name: string | null;
	position: string | null;
	member_id: string | null;
}

interface Props {
    postgrest: PostgrestClient;
}

interface State {
    memberData: Member | null;
    isLoading: boolean;
    initialDataLoaded: boolean;
    isUpdating: boolean;
}

const DetailMemberPage = (props: Props) => {

    const [state, setState] = React.useState<State>({
        memberData: null,
        isLoading: false,
        initialDataLoaded: false,
        isUpdating: false,
    });

    const params = useParams();


    React.useEffect(() => {
        if (state.initialDataLoaded !== false || state.isLoading) {
            return;
        }
        setState({ ...state, isLoading: true });
    
        //todo: add loading of address (should be already in same table?) and occupation from other db table
        props.postgrest
            .from("members")
            .select('*')
            .eq('member_number', params.id)
            .then((result: PostgrestSingleResponse<any[]>) => {
                if (result.data !== null) {
                    console.log(result.data)
                    setState({
                        ...state,
                        initialDataLoaded: true,
                        memberData: result.data[0],
                        isLoading: false,
                    });
                } else {
                    setState({ ...state, isLoading: false });
                }
            });
    })

    const onUpdateMemberClick = (updatedMember: Member) => {
        console.log(updatedMember)
        
        //todo: test this implementation
        if (state.isUpdating) {
            return;
        }

        setState({ ...state, isUpdating: true });

        props.postgrest.from('members').upsert(updatedMember)
            .then((result: PostgrestSingleResponse<null>) => {
                console.log(result);
                
                if (result.error) {
                    setState({ ...state, isUpdating: false })
                    console.log(result.error);
                    //todo: display error
                    return;
                }
                
                setState({ ...state, isUpdating: false })
                
        })
    };
    
    const onUpdateAddressClick = (updatedAddress: Member) => {
        console.log(updatedAddress)
        
        //todo: implement update of address
        if (state.isUpdating) {
            return;
        }
    };

    const onUpdateOccupationClick = (updatedOccupation: Occupation) => {
        console.log(updatedOccupation)
        
        //todo: implement update of occupation
        if (state.isUpdating) {
            return;
        }
    };

    //todo: implement some UI
    // my idea would be:
    // editable textboxes with member info
    // button to update member
    // editable textboxes with address info
    // button to update address
    // editable textboxes with occupation info
    // button to update occupation
    return (
        <div>
            <h2>Member detail</h2>
            <div>{JSON.stringify(state.memberData)}</div>
        </div>
    )
    
}

export default DetailMemberPage

