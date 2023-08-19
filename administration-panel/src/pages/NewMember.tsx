import * as React from 'react';
import { Button, TextField, MenuItem } from '@mui/material';
import { Form } from 'react-router-dom';
import Box from '@mui/material/Box';

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
    newMember: Member;
    newMemberOccupation: Occupation;
    isUpdating: boolean;

}

const languages = [
    "cs", "en"
];

export default class NewMember extends React.Component<Props, State> {

    state: State = {
        newMember: { 
            member_number: null, email: null, first_name: null, last_name: null,
            date_of_birth: null, address: null, city: null, postal_code: null,
            language: languages[0], phone_number: null,     
        },
        newMemberOccupation: {
            company_name: null, position: null, member_id: null,
        },
        isUpdating: false,
    }


    setMemberNumber(event: React.ChangeEvent<HTMLInputElement>) {
        this.setState((state) => {
            const member = state.newMember;
            return { ...state, newMember: { ...member, member_number: event.target.value ? event.target.valueAsNumber : null}}
        });
    }

    setFirstName(event: React.ChangeEvent<HTMLInputElement>) {
        this.setState((state) => {
            const member = state.newMember;
            return { ...state, newMember: { ...member, first_name: event.target.value}}
        });
    }

    setLastName(event: React.ChangeEvent<HTMLInputElement>) {
        this.setState((state) => {
            const member = state.newMember;
            return { ...state, newMember: { ...member, last_name: event.target.value}}
        });
    }

    setEmail(event: React.ChangeEvent<HTMLInputElement>) {
        this.setState((state) => {
            const member = state.newMember;
            return { ...state, newMember: { ...member, email: event.target.value}}
        });
    }

    setDateOfBirth(event: React.ChangeEvent<HTMLInputElement>) {
        this.setState((state) => {
            const member = state.newMember;
            return { ...state, newMember: { ...member, date_of_birth: event.target.valueAsDate}}
        });
    }

    setAddress(event: React.ChangeEvent<HTMLInputElement>) {
        this.setState((state) => {
            const member = state.newMember;
            return { ...state, newMember: { ...member, address: event.target.value}}
        });
    }

    setCity(event: React.ChangeEvent<HTMLInputElement>) {
        this.setState((state) => {
            const member = state.newMember;
            return { ...state, newMember: { ...member, city: event.target.value}}
        });
    }

    setPostalCode(event: React.ChangeEvent<HTMLInputElement>) {
        this.setState((state) => {
            const member = state.newMember;
            return { ...state, newMember: { ...member, postal_code: event.target.value}}
        });
    }

    setLanguage(event: React.ChangeEvent<HTMLInputElement>) {
        this.setState((state) => {
            const member = state.newMember;
            return { ...state, newMember: { ...member, language: event.target.value}}
        });
    }

    setPhoneNumber(event: React.ChangeEvent<HTMLInputElement>) {
        this.setState((state) => {
            const member = state.newMember;
            return { ...state, newMember: { ...member, phone_number: event.target.value}}
        });
    }

    setCompanyName(event: React.ChangeEvent<HTMLInputElement>) {
        this.setState((state) => {
            const occupation = state.newMemberOccupation;
            return { ...state, newMemberOccupation: { ...occupation, company_name: event.target.value}}
        });
    }
    
    setPosition(event: React.ChangeEvent<HTMLInputElement>) {
        this.setState((state) => {
            const occupation = state.newMemberOccupation;
            return { ...state, newMemberOccupation: { ...occupation, position: event.target.value}}
        });
    }

    setMemberIdToOccupation(memberId: string) {
        this.setState((state) => {
            const occupation = state.newMemberOccupation;
            return { ...state, newMemberOccupation: { ...occupation, member_id: memberId}}
        });
    }

    saveNewMember() {
        //TODO: add validation?
        
        if (this.state.isUpdating) {
            return;
        }

        this.setState({ ...this.state, isUpdating: true });

        this.props.postgrest.from('members').insert(this.state.newMember)
            .then((result: PostgrestSingleResponse<null>) => {
                console.log(result);
                
                if (result.error) {
                    this.setState({ ...this.state, isUpdating: false })
                    console.log(result.error);
                    //todo: display error
                    return;
                }

                this.props.postgrest.from('members').select('id').eq('member_number', this.state.newMember.member_number)
                .then((result: PostgrestSingleResponse<any[]>) => {

                    if (result.error || result.data == null) {
                        this.setState({ ...this.state, isUpdating: false })
                        console.log(result.error);
                        //todo: display error
                        return;
                    }

                    let occupationWithMemberId = this.state.newMemberOccupation
                    occupationWithMemberId.member_id = result.data[0].id

                    this.props.postgrest.from('occupations').insert(occupationWithMemberId)
                        .then((result: PostgrestSingleResponse<null>) => {
                            console.log(result);

                            if (result.error) {
                                this.setState({ ...this.state, isUpdating: false })
                                console.log(result.error);
                                //todo: display error
                                return;
                            }

                            this.setState({ ...this.state, isUpdating: false })
                    })

                })
                
        })
    }

    render(): React.ReactNode {
        return (
            <div>
                <h2>Add New Member</h2>
                <Box
                    component="form"
                    sx={{
                        '& .MuiTextField-root': { m: 1, width: '25ch' },
                    }}
                    noValidate
                    autoComplete="off" >
                        <div>
                            <TextField
                                required
                                
                                label="Member number"
                                type="number"
                                value={this.state.newMember.member_number}
                                onInput={this.setMemberNumber.bind(this)}
                            /> 
                            <TextField
                                required
                                
                                label="First name"
                                value={this.state.newMember.first_name}
                                onInput={this.setFirstName.bind(this)}
                            />
                            <TextField
                                required
                                
                                label="Last name"
                                value={this.state.newMember.last_name}
                                onInput={this.setLastName.bind(this)}
                            />
                            <TextField
                                required
                                
                                label="Email"
                                value={this.state.newMember.email}
                                onInput={this.setEmail.bind(this)}
                            />
                        </div>
                        <div>
                            <TextField
                                required
                                
                                label="Date of Birth"
                                type="date"
                                onInput={this.setDateOfBirth.bind(this)}
                                InputLabelProps={{
                                    shrink: true
                                }}
                            />
                            <TextField
                                required
                                
                                label="Address"
                                value={this.state.newMember.address}
                                onInput={this.setAddress.bind(this)}
                            />
                            <TextField
                                required
                                
                                label="City"
                                value={this.state.newMember.city}
                                onInput={this.setCity.bind(this)}
                            />
                            <TextField
                                required
                                
                                label="Postal Code"
                                value={this.state.newMember.postal_code}
                                onInput={this.setPostalCode.bind(this)}
                            />
                        </div>
                        <div>
                            <TextField
                                required
                                
                                label="Language"
                                select
                                value={this.state.newMember.language}
                                onInput={this.setLanguage.bind(this)}>

                                {languages.map((option) => (
                                    <MenuItem key={option} value={option}>
                                    {option}
                                    </MenuItem>
                                ))}


                            </TextField>
                            
                            <TextField
                                required
                                
                                label="Phone number"
                                value={this.state.newMember.phone_number}
                                onInput={this.setPhoneNumber.bind(this)}
                            />

                            <TextField
                                required
                                
                                label="Company"
                                value={this.state.newMemberOccupation.company_name}
                                onInput={this.setCompanyName.bind(this)}
                            />

                            <TextField  
                                required
                                
                                label="Occupation"
                                value={this.state.newMemberOccupation.position}
                                onInput={this.setPosition.bind(this)}
                            />
                        </div>
                </Box>
                <Button variant="contained" onClick={this.saveNewMember.bind(this)}>Save new member</Button>
            </div>
        )
    }
}
