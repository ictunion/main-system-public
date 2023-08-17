import * as React from 'react';
import {
    Divider, Box, AppBar, Toolbar, IconButton, Typography, Menu, MenuItem, Drawer
} from '@mui/material';
import { Menu as MenuIcon, AccountCircle } from '@mui/icons-material';

interface Props {
    logout: () => void;
}

const openMembersPanel = () => {
    window.location.href = 'https://member.ictunion.cz'
};

const AppHeader: React.FC<React.PropsWithChildren<Props>> = (props) => {
    // acount dropdown menu state
    const [anchorEl, setAnchorEl] = React.useState<null | HTMLElement>(null);
    const handleProfileMenuOpen = (event: React.MouseEvent<HTMLElement>) => {
        setAnchorEl(event.currentTarget);
    };
    const handleMenuClose = () => {
        setAnchorEl(null);
    };

    // Drawer / navigation state
    const [drawerState, setDrawerState] = React.useState(false);

    const toggleDrawer = (val: boolean) => (event: React.KeyboardEvent | React.MouseEvent) => {
        if (
            event.type === 'keydown' &&
            ((event as React.KeyboardEvent).key === 'Tab' ||
                (event as React.KeyboardEvent).key === 'Shift')
        ) {
            return;
        }

        setDrawerState(val);
    };

    // Dropdown menu
    const isMenuOpen = Boolean(anchorEl);
    const menuId = 'primary-account-menu';
    const renderMenu = (
        <Menu
            anchorEl={anchorEl}
            anchorOrigin={{
                vertical: 'top',
                horizontal: 'right',
            }}
            id={menuId}
            keepMounted
            transformOrigin={{
                vertical: 'top',
                horizontal: 'right',
            }}
            open={isMenuOpen}
            onClose={handleMenuClose}
        >
            <MenuItem onClick={openMembersPanel}>Profile</MenuItem>
            <Divider />
            <MenuItem onClick={props.logout}>Logout</MenuItem>
        </Menu>
    );

    return (
        <Box sx={{ flexGrow: 1 }}>
            <AppBar position="static">
                <Toolbar>
                    <IconButton
                        edge="start"
                        color="inherit"
                        aria-label="menu"
                        onClick={toggleDrawer(true)}
                        sx={{ mr: 2 }}>
                        <MenuIcon />
                    </IconButton>
                    <Typography variant="h6" color="inherit" component="div" sx={{ flexGrow: 1 }}>
                        Administration Panel
                    </Typography>
                    <IconButton
                        size="large"
                        edge="end"
                        aria-label="account of current user"
                        aria-haspopup="true"
                        onClick={handleProfileMenuOpen}
                        color="inherit"
                    >
                        <AccountCircle />
                    </IconButton>
                </Toolbar>
            </AppBar>
            {renderMenu}
            <Drawer
                anchor='left'
                open={drawerState}
                onClose={toggleDrawer(false)}
            >
                <Box
                    sx={{ width: 250 }}
                    role="presentation"
                    onClick={toggleDrawer(false)}
                    onKeyDown={toggleDrawer(false)}
                >
                    {props.children}
                </Box>
            </Drawer>
        </Box>
    )
}

export default AppHeader;
