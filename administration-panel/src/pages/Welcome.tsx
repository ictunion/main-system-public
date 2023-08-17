import * as React from 'react';
import { UserInfo } from '@app/keycloak';
import { Box } from '@mui/material';

interface Props {
    userInfo: UserInfo | null;
}

const page: React.FC<Props> = (props) => {
    return (
        <Box sx={{ flexGrow: 1 }}>
            <Box sx={{ flexGrow: 1 }}>
                Hello <strong>{props.userInfo && props.userInfo.preferred_username}!</strong><br />
            </Box>
            <pre>
                {JSON.stringify(props.userInfo)}
            </pre>
        </Box>
    )
}

export default page;
