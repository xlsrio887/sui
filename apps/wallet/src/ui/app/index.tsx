// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

import { useEffect } from 'react';
import { Navigate, Outlet, Route, Routes, useLocation } from 'react-router-dom';

import { useInitialPageView } from './hooks/useInitialPageView';
import { useStorageMigrationStatus } from './hooks/useStorageMigrationStatus';
import { StorageMigrationPage } from './pages/StorageMigrationPage';

import { AccountsPage } from './pages/accounts/AccountsPage';
import { AddAccountPage } from './pages/accounts/AddAccountPage';
import { ForgotPasswordPage } from './pages/accounts/ForgotPasswordPage';
import { ImportLedgerAccountsPage } from './pages/accounts/ImportLedgerAccountsPage';
import { ImportPassphrasePage } from './pages/accounts/ImportPassphrasePage';
import { ImportPrivateKeyPage } from './pages/accounts/ImportPrivateKeyPage';
import { ManageAccountsPage } from './pages/accounts/manage/ManageAccountsPage';
import { ProtectAccountPage } from './pages/accounts/ProtectAccountPage';
import { AccountsDev } from './pages/accounts-dev';

import { ApprovalRequestPage } from './pages/approval-request';
import HomePage, {
	AppsPage,
	AssetsPage,
	CoinsSelectorPage,
	KioskDetailsPage,
	NFTDetailsPage,
	NftTransferPage,
	OnrampPage,
	ReceiptPage,
	TransactionBlocksPage,
	TransferCoinPage,
} from './pages/home';
import TokenDetailsPage from './pages/home/tokens/TokenDetailsPage';
import InitializePage from './pages/initialize';
import BackupPage from './pages/initialize/backup';
import CreatePage from './pages/initialize/create';
import { ImportPage } from './pages/initialize/import';
import SelectPage from './pages/initialize/select';
import { QredoConnectInfoPage } from './pages/qredo-connect/QredoConnectInfoPage';
import { SelectQredoAccountsPage } from './pages/qredo-connect/SelectQredoAccountsPage';

import { RestrictedPage } from './pages/restricted';
import SiteConnectPage from './pages/site-connect';

import { AppType } from './redux/slices/app/AppType';
import { Staking } from './staking/home';
import LockedPage from './wallet/locked-page';

import { useAppDispatch, useAppSelector } from '_hooks';

import { setNavVisibility } from '_redux/slices/app';
import { NEW_ACCOUNTS_ENABLED } from '_src/shared/constants';
import { WelcomePage } from './pages/zklogin/WelcomePage';

const HIDDEN_MENU_PATHS = [
	'/nft-details',
	'/nft-transfer',
	'/receipt',
	'/send',
	'/send/select',
	'/apps/disconnectapp',
];

const App = () => {
	const dispatch = useAppDispatch();
	const isPopup = useAppSelector((state) => state.app.appType === AppType.popup);
	useEffect(() => {
		document.body.classList.remove('app-initializing');
	}, [isPopup]);
	const location = useLocation();
	useEffect(() => {
		const menuVisible = !HIDDEN_MENU_PATHS.some((aPath) => location.pathname.startsWith(aPath));
		dispatch(setNavVisibility(menuVisible));
	}, [location, dispatch]);

	useInitialPageView();

	const storageMigration = useStorageMigrationStatus();
	if (storageMigration.isLoading || !storageMigration?.data) {
		return null;
	}
	if (storageMigration.data !== 'ready') {
		return <StorageMigrationPage />;
	}

	return (
		<Routes>
			<Route path="/welcome" element={<WelcomePage />} />

			<Route path="locked" element={<LockedPage />} />
			<Route path="forgot-password" element={<ForgotPasswordPage />} />
			<Route path="restricted" element={<RestrictedPage />} />

			<Route path="/initialize" element={<InitializePage />}>
				<Route path="select" element={<SelectPage />} />
				<Route path="create" element={<CreatePage />} />
				<Route path="import" element={<ImportPage />} />
				<Route path="backup" element={<BackupPage />} />
				<Route path="backup-imported" element={<BackupPage mode="imported" />} />
			</Route>

			<Route path="/*" element={<HomePage />}>
				<Route path="apps/*" element={<AppsPage />} />
				<Route path="kiosk" element={<KioskDetailsPage />} />
				<Route path="nft-details" element={<NFTDetailsPage />} />
				<Route path="nft-transfer/:nftId" element={<NftTransferPage />} />
				<Route path="nfts/*" element={<AssetsPage />} />
				<Route path="onramp" element={<OnrampPage />} />
				<Route path="receipt" element={<ReceiptPage />} />
				<Route path="send" element={<TransferCoinPage />} />
				<Route path="send/select" element={<CoinsSelectorPage />} />
				<Route path="stake/*" element={<Staking />} />
				<Route path="tokens/*" element={<TokenDetailsPage />} />
				<Route path="transactions/:status?" element={<TransactionBlocksPage />} />
				<Route path="*" element={<Navigate to="/tokens" replace={true} />} />
			</Route>

			<Route path="accounts/*" element={<AccountsPage />}>
				<Route path="add-account" element={<AddAccountPage />} />
				<Route path="add-account" element={<AddAccountPage />} />
				<Route path="import-ledger-accounts" element={<ImportLedgerAccountsPage />} />
				<Route path="import-passphrase" element={<ImportPassphrasePage />} />
				<Route path="import-private-key" element={<ImportPrivateKeyPage />} />
				<Route path="manage" element={<ManageAccountsPage />} />
				<Route path="protect-account" element={<ProtectAccountPage />} />
			</Route>

			<Route path="/account">
				<Route path="forgot-password" element={<ForgotPasswordPage />} />
			</Route>

			<Route path="/dapp/*" element={<HomePage disableNavigation />}>
				<Route path="connect/:requestID" element={<SiteConnectPage />} />
				<Route path="approve/:requestID" element={<ApprovalRequestPage />} />
				<Route path="qredo-connect/:requestID" element={<QredoConnectInfoPage />} />
				<Route path="qredo-connect/:id/select" element={<SelectQredoAccountsPage />} />
			</Route>

			{/* this is used only for making dev work on refactoring accounts easier - TODO: remove when work is done ----> */}
			{process.env.NODE_ENV === 'development' && NEW_ACCOUNTS_ENABLED ? (
				<>
					<Route path="/accounts-dev" element={<AccountsDev />} />
					<Route
						path="/dapp/"
						element={
							<>
								<div className="p-3 flex bg-white rounded-lg flex-col w-80">
									<Outlet />
								</div>
								<div id="overlay-portal-container"></div>
							</>
						}
					>
						<Route path="/dapp/qredo-connect/:requestID" element={<QredoConnectInfoPage />} />
						<Route path="/dapp/qredo-connect/:id/select" element={<SelectQredoAccountsPage />} />
					</Route>
				</>
			) : null}
		</Routes>
	);
};

export default App;
