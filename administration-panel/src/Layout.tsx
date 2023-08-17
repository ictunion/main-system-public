import * as React from 'react';
import AppHeader from '@app/layout/AppHeader';
import { Outlet } from "react-router-dom";
import { Box, List, ListItem, ListItemButton, ListItemIcon, ListItemText } from '@mui/material';
import { Link } from 'react-router-dom';
import { Home as HomeIcon } from '@mui/icons-material';

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
        <div style={{ minHeight: "100%" }}>
            <AppHeader logout={props.logout}>
                <List>
                    <MenuItem text="Home" route="/"><HomeIcon /></MenuItem>
                </List>
            </AppHeader>
            <Box sx={{ flexGrow: 1 }}>
                <Outlet />
            </Box>
        </div>
    );
}

export default Layout;
