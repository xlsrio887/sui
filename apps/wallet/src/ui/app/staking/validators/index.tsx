// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

import Alert from '_components/alert';
import Loading from '_components/loading';
import Overlay from '_components/overlay';
import { useNavigate } from 'react-router-dom';

import { useActiveAddress } from '../../hooks/useActiveAddress';
import { useGetDelegatedStake } from '../useGetDelegatedStake';
import { SelectValidatorCard } from './SelectValidatorCard';
import { ValidatorsCard } from './ValidatorsCard';

export function Validators() {
	const accountAddress = useActiveAddress();
	const {
		data: stakedValidators,
		isPending,
		isError,
		error,
	} = useGetDelegatedStake(accountAddress || '');

	const navigate = useNavigate();

	const pageTitle = stakedValidators?.length ? 'Stake & Earn SUI' : 'Select a Validator';

	return (
		<Overlay showModal title={isPending ? 'Loading' : pageTitle} closeOverlay={() => navigate('/')}>
			<div className="w-full h-full flex flex-col flex-nowrap">
				<Loading loading={isPending}>
					{isError ? (
						<div className="mb-2">
							<Alert>
								<strong>{error?.message}</strong>
							</Alert>
						</div>
					) : null}

					{stakedValidators?.length ? <ValidatorsCard /> : <SelectValidatorCard />}
				</Loading>
			</div>
		</Overlay>
	);
}
