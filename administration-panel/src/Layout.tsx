import * as React from 'react';
import AppHeader from '@app/layout/AppHeader';
import { Outlet } from "react-router-dom";
import { Box, Divider, List, ListItem, ListItemButton, ListItemIcon, ListItemText } from '@mui/material';
import { Link } from 'react-router-dom';
import {
    Home as HomeIcon,
    PeopleAlt as PeopleAltIcon,
    PersonAddAlt1 as PersonAddIcon
} from '@mui/icons-material';

interface Props {
    logout: () => void;
}

interface ItemProps {
    text: string;
    route: string;
}

const MenuItem = ({ text, route, children }: React.PropsWithChildren<ItemProps>) => {
    return (
        <Link to={route} style={{ color: "inherit", textDecoration: "none" }}>
            <ListItem key={text} disablePadding>
                <ListItemButton>
                    <ListItemIcon>
                        {children}
                    </ListItemIcon>
                    <ListItemText primary={text} />
                </ListItemButton>
            </ListItem>
        </Link>
    )
}

const Layout = (props: Props) => {
    return (
        <div className="app-wrapper">
            <AppHeader logout={props.logout}>
                <List>
                    <MenuItem text="Dashboard" route="/"><HomeIcon /></MenuItem>
                    <Divider />
                    <MenuItem text="Members Table" route="/members/table"><PeopleAltIcon /></MenuItem>
                    <MenuItem text="Add New Member" route="/members/new"><PersonAddIcon /></MenuItem>
                    <Divider />
                </List>
            </AppHeader>
            <Box className="app-main">
                <Outlet />
            </Box>
        </div>
    );
}

export default Layout;
